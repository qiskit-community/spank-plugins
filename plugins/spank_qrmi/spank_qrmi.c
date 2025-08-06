/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM 2025
 *
 * This program and the accompanying materials are made available under the
 * terms of the GNU General Public License version 3, as published by the
 * Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see
 * <[https://www.gnu.org/licenses/gpl-3.0.txt]
 */
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "slurm/slurm.h"
#include "slurm/spank.h"

#include "qrmi.h"
#include "spank_qrmi.h"

/*
 * Spank plugin for QRMI.
 */
SPANK_PLUGIN(spank_qrmi_c, 1)

/*
 * Copy of `--qpu` option value if specified, otherwise NULL.
 */
static char *g_qpu_names_opt = NULL;

/*
 * List of the acquired QPU resources.
 */
#ifndef PRIOR_TO_V24_05_5_1
static list_t *g_acquired_resources = NULL;
#else
static List g_acquired_resources = NULL;
#endif /* !PRIOR_TO_V24_05_5_1 */

/*
 *  Forward declarations
 */
static qpu_resource_t *_acquired_resource_create(char *name, QrmiResourceType type,
                                                 const char *token);
static void acquired_resource_destroy(void *object);
static qpu_resource_t *_acquire_qpu(char *name, QrmiResourceType type);
static void _release_qpu(qpu_resource_t *res);

/*
 * @function _qpu_names_opt_cb
 *
 * A callback function that is invoked when the `--qpu` plugin option is registered with Slurm.
 */
static int _qpu_names_opt_cb(int val, const char *optarg, int remote) {
    size_t buflen = strlen(optarg) + 1;
    g_qpu_names_opt = (char *)malloc(buflen);
    strncpy(g_qpu_names_opt, optarg, buflen);
    slurm_debug("%s: --qpu=[%s]", plugin_name, g_qpu_names_opt);
    return ESPANK_SUCCESS;
}

/*
 * Options available to this spank plugin.
 */
struct spank_option spank_qrmi_options[] = {
    {"qpu", "names", "Comma separated list of QPU resources to use.",
     1, /* argument is required */
     0, /* value to return using callback */
     (spank_opt_cb_f)_qpu_names_opt_cb},
    SPANK_OPTIONS_TABLE_END};

/*
 * @function slurm_spank_init
 *
 * Called just after plugins are loaded. In remote context, this is just after
 * job step is initialized. This function is called before any plugin option
 * processing.
 *
 */
int slurm_spank_init(spank_t spank_ctxt, int argc, char *argv[]) {
    int rc = ESPANK_SUCCESS;
    struct spank_option *opts_to_register = NULL;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid,
                __FUNCTION__, argc, spank_remote(spank_ctxt));

    g_acquired_resources = slurm_list_create(acquired_resource_destroy);

    /*
     * Get any options registered for this context:
     */
    switch (spank_context()) {
    /* salloc, sbatch */
    case S_CTX_ALLOCATOR:
    /* srun */
    case S_CTX_LOCAL:
    /* slurmstepd */
    case S_CTX_REMOTE:
        opts_to_register = spank_qrmi_options;
        break;

    default:
        break;
    }
    if (opts_to_register) {
        while (opts_to_register->name && (rc == ESPANK_SUCCESS)) {
            rc = spank_option_register(spank_ctxt, opts_to_register++);
        }
    }
    slurm_debug("%s(%d,%d): <- %s rc=%d", plugin_name, pid, uid, __FUNCTION__,
                rc);
    return rc;
}

/*
 * @function slurm_spank_init_post_opt
 *
 * Called at the same point as slurm_spank_init, but after all user options to
 * the plugin have been processed. The reason that the init and init_post_opt
 * callbacks are separated is so that plugins can process system-wide options
 * specified in plugstack.conf in the init callback, then process user options,
 * and finally take some action in slurm_spank_init_post_opt if necessary. In
 * the case of a heterogeneous job, slurm_spank_init is invoked once per job
 * component.
 *
 */
int slurm_spank_init_post_opt(spank_t spank_ctxt, int argc, char **argv) {
    int rc = ESPANK_SUCCESS;
    uint32_t job_id = 0;
    uint32_t job_stepid = 0;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid,
                __FUNCTION__, argc, spank_remote(spank_ctxt));

    if (!spank_remote(spank_ctxt)) {
        /* never occurred. just for safe. */
        return rc;
    }

    if (spank_get_item(spank_ctxt, S_JOB_STEPID, &job_stepid) ==
        ESPANK_SUCCESS) {
        /* skip if this is slurm task steps */
        slurm_debug("%s, %x", plugin_name, job_stepid);
        if (job_stepid != SLURM_BATCH_SCRIPT) {
            return rc;
        }
    }

    if (g_qpu_names_opt == NULL) {
        /* noop if this is not QPU job */
        return ESPANK_SUCCESS;
    }

    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_RESOURCES", "", OVERWRITE);
    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_TYPES", "", OVERWRITE);

    for (int i = 0; i < argc; i++) {
        slurm_debug("%s: argv[%d] = [%s]", plugin_name, i, argv[i]);
    }

    QrmiConfig *cnf = qrmi_config_load(argv[0]);
    slurm_debug("%s, config: %p", plugin_name, cnf);

    char *bufp = (char *)malloc(strlen(g_qpu_names_opt) + 1);
    char *rest = bufp;
    char *token;
    buffer keybuf;
    qrmi_buf_init(&keybuf, 1024);

    /*
     * Copy option string to bufp because subsequent strtok_r will
     * modify the source buffer.
     */
    strncpy(bufp, g_qpu_names_opt,
            strlen(g_qpu_names_opt) + 1); /* ends with '\0' */

    while ((token = strtok_r(rest, ",", &rest))) {
        QrmiResourceDef *res = qrmi_config_resource_def_get(cnf, token);
        if (res != NULL) {
            slurm_debug("%s: name(%s), type(%d) found in qrmi_config",
                        plugin_name, res->name, res->type);
            /*
             * If user specifies access details in environment variables,
             * these are available as job environment variables. Reads through
             * them and set user-specified {qpu_name}_QRMI_xxx env vars to this
             * slurm daemon process for subsequent QRMI.acquire/release call.
             */
            qrmi_buf_envvarname_for_res_create(&keybuf, res->name, "QRMI_");
            char **job_argv = NULL;
            if (spank_get_item(spank_ctxt, S_JOB_ENV, &job_argv) ==
                ESPANK_SUCCESS) {
                int index = 0;
                while (job_argv[index] != NULL) {
                    if (strncmp(job_argv[index], keybuf.buffer,
                                strlen(keybuf.buffer)) == 0) {
                        putenv(job_argv[index]);
                        slurm_debug("%s: putenv(%s)", plugin_name, job_argv[index]);
                    }
                    index++;
                }
            }

            /*
             * Next, set environment variables specified in config file.
             */
            QrmiEnvironmentVariables envvars = res->environments;
            for (int j = 0; j < envvars.length; j++) {
                QrmiKeyValue envvar = envvars.variables[j];
                /* set to the current process for subsequent QRMI.acquire() call
                 */
                qrmi_buf_envvarname_for_res_create(&keybuf, res->name,
                                                   envvar.key);
                slurm_debug("%s: setenv(%s, %s)", plugin_name, keybuf.buffer,
                            envvar.value);
                setenv(keybuf.buffer, envvar.value, KEEP_IF_EXISTS);
                spank_setenv(spank_ctxt, keybuf.buffer, envvar.value,
                             KEEP_IF_EXISTS);
            }

            /*
             * Acquire QPU resource.
             */
            qpu_resource_t *acquired = _acquire_qpu(res->name, res->type);
            if (acquired != NULL) {
                slurm_list_append(g_acquired_resources, acquired);
                qrmi_buf_envvarname_for_res_create(&keybuf, res->name,
                                                   "QRMI_JOB_ACQUISITION_TOKEN");
                slurm_debug("%s: setenv(%s, %s)", plugin_name,
                            keybuf.buffer, acquired->acquisition_token);
                setenv(keybuf.buffer, acquired->acquisition_token,
                       KEEP_IF_EXISTS);
                spank_setenv(spank_ctxt, keybuf.buffer,
                             acquired->acquisition_token, KEEP_IF_EXISTS);
            } else {
                slurm_error("%s, failed to acquire resource: %s", plugin_name,
                            res->name);
            }
            qrmi_config_resource_def_free(res);
        } else {
            slurm_debug("resource %s not found.", token);
        }
    }
    free(bufp);
    qrmi_buf_free(&keybuf);

    if (slurm_list_count(g_acquired_resources) == 0) {
        slurm_error("%s, No QPU resource available", plugin_name);
        return ESPANK_ERROR;
    }

    string_buffer_t qpu_resources_envvar;
    string_buffer_t qpu_types_envvar;
    strbuf_init(&qpu_resources_envvar);
    strbuf_init(&qpu_types_envvar);
#ifndef PRIOR_TO_V24_05_5_1
    list_itr_t *sessions_iter =
#else
    ListIterator sessions_iter =
#endif /* !PRIOR_TO_V24_05_5_1 */
        slurm_list_iterator_create(g_acquired_resources);
    void *x = NULL;
    while ((x = slurm_list_next(sessions_iter)) != NULL) {
        qpu_resource_t *item = (qpu_resource_t *)x;
        slurm_debug("%s: name(%s), type(%d), token(%s)", plugin_name,
                    item->name, item->type, item->acquisition_token);
        strbuf_append_str(&qpu_resources_envvar, item->name);
        const char* type_as_str = qrmi_config_resource_type_to_str(item->type);
        slurm_debug("%s: type_as_str(%s)", plugin_name, type_as_str);
        strbuf_append_str(&qpu_types_envvar, qrmi_config_resource_type_to_str(item->type));
        qrmi_string_free((char*)type_as_str);
    }
    slurm_list_iterator_destroy(sessions_iter);

    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_RESOURCES",
                 qpu_resources_envvar.buffer, OVERWRITE);
    slurm_debug("%s: setenv(%s, %s)", plugin_name,
                "SLURM_JOB_QPU_RESOURCES", qpu_resources_envvar.buffer);
    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_TYPES", qpu_types_envvar.buffer,
                 OVERWRITE);
    slurm_debug("%s: setenv(%s, %s)", plugin_name,
                "SLURM_JOB_QPU_TYPES", qpu_types_envvar.buffer);
    strbuf_free(&qpu_resources_envvar);
    strbuf_free(&qpu_types_envvar);

    slurm_debug("%s(%d,%d): <- %s rc=%d", plugin_name, pid, uid, __FUNCTION__,
                rc);
    return rc;
}

/*
 * @function slurm_spank_task_init
 *
 * Called for each task just before execve (2). If you are restricting memory
 * with cgroups, memory allocated here will be in the job's cgroup. (remote
 * context only)
 */
int slurm_spank_task_init(spank_t spank_ctxt, int argc, char **argv) {
    int rc = ESPANK_SUCCESS;
    uint32_t job_id = 0;
    job_info_msg_t *job_info_msg = NULL;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid,
                __FUNCTION__, argc, spank_remote(spank_ctxt));

    if (!spank_remote(spank_ctxt)) {
        /* never occurred. just for safe. */
        return ESPANK_SUCCESS;
    }

    char *optargp = NULL;
    rc = spank_option_getopt(spank_ctxt, &spank_qrmi_options[0], &optargp);
    if (rc != ESPANK_SUCCESS) {
        /* if spank_qrmi plugin is not registered, simply returns an error. */
        return rc;
    }
    size_t optlen = strlen(optargp);
    if ((optargp == NULL) || optlen == 0) {
        /* noop if this is not QPU job */
        return ESPANK_SUCCESS;
    }

    char limit_as_str[11]; /* max uint32_t value is (2147483647) = 10 chars */
    memset(limit_as_str, '\0', sizeof(limit_as_str));
    if (spank_get_item(spank_ctxt, S_JOB_ID, &job_id) == ESPANK_SUCCESS) {
        if (slurm_load_job(&job_info_msg, job_id, SHOW_DETAIL) ==
            SLURM_SUCCESS) {
            /*
             * Slurm's time limit is represented in minutes
             */
            uint32_t time_limit_mins = job_info_msg->job_array[0].time_limit;
            /*
             * Convert minutes to seconds, uint32_t to char*
             */
            memset(limit_as_str, '\0', sizeof(limit_as_str));
            snprintf(limit_as_str, sizeof(limit_as_str), "%u",
                     time_limit_mins * 60);
        }
    }

    if (strlen(limit_as_str) == 0) {
        /* time limit should be there, somthing wrong in Slurm */
        return ESPANK_ERROR;
    }

    /*
     * Set environment variables for timeout -
     * {qpu_name}_QRMI_JOB_TIMEOUT_SECONDS=val
     */
    buffer keybuf;
    qrmi_buf_init(&keybuf, 1024);
#ifndef PRIOR_TO_V24_05_5_1
    list_itr_t *resources_iter =
#else    
    ListIterator resources_iter =
#endif /* !PRIOR_TO_V24_05_5_1 */
        slurm_list_iterator_create(g_acquired_resources);
    void *x = NULL;
    while ((x = slurm_list_next(resources_iter)) != NULL) {
        qpu_resource_t *res = (qpu_resource_t *)x;
        qrmi_buf_envvarname_for_res_create(&keybuf, res->name,
                                           "QRMI_JOB_TIMEOUT_SECONDS");
        spank_setenv(spank_ctxt, keybuf.buffer, limit_as_str, OVERWRITE);
        slurm_debug("%s: setenv(%s, %s)", plugin_name, keybuf.buffer,
                    limit_as_str);
    }
    slurm_list_iterator_destroy(resources_iter);
    qrmi_buf_free(&keybuf);

    slurm_debug("%s(%d,%d): <- %s rc=%d", plugin_name, pid, uid, __FUNCTION__,
                rc);

    return rc;
}

/*
 * @function slurm_spank_exit
 *
 * Called once just before slurmstepd exits in remote context. In local
 * context, called before srun exits.
 */
int slurm_spank_exit(spank_t spank_ctxt, int argc, char **argv) {
    int rc = ESPANK_SUCCESS;
    int pid = (int)getpid();
    int uid = (int)getuid();

    if (!spank_remote(spank_ctxt)) {
        /* never occurred. just for safe. */
        return ESPANK_SUCCESS;
    }

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid,
                __FUNCTION__, argc, spank_remote(spank_ctxt));

#ifndef PRIOR_TO_V24_05_5_1
    list_itr_t *resources_iter =
#else
    ListIterator resources_iter =
#endif /* !PRIOR_TO_V24_05_5_1 */
        slurm_list_iterator_create(g_acquired_resources);
    void *x = NULL;
    while ((x = slurm_list_next(resources_iter)) != NULL) {
        _release_qpu((qpu_resource_t *)x);
    }
    slurm_list_iterator_destroy(resources_iter);
    slurm_list_destroy(g_acquired_resources);

    g_acquired_resources = NULL;

    if (g_qpu_names_opt != NULL) {
        free(g_qpu_names_opt);
        g_qpu_names_opt = NULL;
    }

    slurm_debug("%s(%d,%d): <- %s rc=%d", plugin_name, pid, uid, __FUNCTION__,
                rc);
    return rc;
}

/*
 * @function acquired_resource_rec
 *
 * Constructs an acquired QPU resource record. See
 * acquired_resource_destroy() to free allocated memory.
 */
static qpu_resource_t *_acquired_resource_create(char *name, QrmiResourceType type,
                                                 const char *token) {
    qpu_resource_t *info = malloc(sizeof(qpu_resource_t));
    size_t buflen = strlen(name) + 1;
    char *bufp = (char *)malloc(buflen);
    strncpy(bufp, name, buflen);
    info->name = bufp;
    info->type = type;

    buflen = strlen(token) + 1;
    bufp = (char *)malloc(buflen);
    strncpy(bufp, token, buflen);
    info->acquisition_token = bufp;

    return info;
}

/*
 * @function qrmiacquired_resource_destroy
 *
 * Destroy an acquired QPU resource record.
 */
static void acquired_resource_destroy(void *object) {
    qpu_resource_t *info = (qpu_resource_t *)object;

    free(info->name);
    free(info->acquisition_token);
    free(info);
}

/*
 * @function _acquire_qpu
 *
 * Acquire QPU resource specified by `name` and `type`. Returns
 * qpu_resource_t object if succeeded.
 */
static qpu_resource_t *_acquire_qpu(char *name, QrmiResourceType type) {
    qpu_resource_t *record = NULL;
    char *acquisition_token = NULL;
    bool is_accessible = false;
    QrmiReturnCode rc;

    void *qrmi = qrmi_resource_new(name, type);
    if (qrmi != NULL) {
        slurm_debug("%s, qrmi: %p", plugin_name, qrmi);
        rc = qrmi_resource_is_accessible(qrmi, &is_accessible);
        if ((rc != QRMI_RETURN_CODE_SUCCESS) || (is_accessible == false)) {
            slurm_error("%s, %s is not accessible", plugin_name, name);
            //functbl->free(qrmi);
            qrmi_resource_free(qrmi);
            return NULL;
        }
        rc = qrmi_resource_acquire(qrmi, &acquisition_token);
        if ((rc == QRMI_RETURN_CODE_SUCCESS) && (acquisition_token != NULL)) {
            slurm_debug("%s, acquisition_token: %s", plugin_name,
                        acquisition_token);
            record = _acquired_resource_create(name, type, acquisition_token);

        } else {
            slurm_error("%s, failed to acquire resource: %s", plugin_name,
                        name);
        }
        qrmi_resource_free(qrmi);
    } else {
        slurm_error("%s/%s: Unsupported resource type: %d", plugin_name,
                    __FUNCTION__, type);
    }

    return record;
}

/*
 * @function _release_qpu
 *
 * Release QPU resource which was acquired by _acquired_qpu().
 */
static void _release_qpu(qpu_resource_t *res) {
    QrmiReturnCode rc;

    if (res == NULL) {
        return;
    }
    slurm_debug("%s: releasing name(%s), type(%d), token(%s)", plugin_name,
                res->name, res->type, res->acquisition_token);
    void *qrmi = qrmi_resource_new(res->name, res->type);
    rc = qrmi_resource_release(qrmi, res->acquisition_token);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
        slurm_error("%s: Failed to release acquired resource: name(%s), type(%d), token(%s)",
                    plugin_name,
                    res->name, res->type, res->acquisition_token);
    }
    rc = qrmi_string_free(res->acquisition_token);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
        slurm_error("%s: Failed to free acquisition token string: (%s)",
                    plugin_name,
                    res->acquisition_token);
    }
    rc = qrmi_resource_free(qrmi);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
        slurm_error("%s: Failed to free QrmiQuantumResource handle: (%p)",
                    plugin_name,
                    qrmi);
    }
    res->acquisition_token = NULL;
}
