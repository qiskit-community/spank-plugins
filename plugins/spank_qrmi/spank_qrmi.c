/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM, Pasqal 2026
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
#include <stdarg.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include <inttypes.h>

#include "slurm/slurm.h"
#include "slurm/spank.h"

#include "qrmi.h"
#include "spank_qrmi.h"

extern char **environ;

/*
 * Spank plugin for QRMI.
 */
SPANK_PLUGIN(spank_qrmi, 1)

/*
 * Copy of `--qpu` option value if specified, otherwise NULL.
 */
static char *g_qpu_names_opt = NULL;

/*
 * This flag indicates whether an error occurred in slurm_spank_init_post_opt().
 * Returning SLURM_ERROR from slurm_spank_init_post_opt() does not cancel the job;
 * instead, it causes the node to be drained. To cancel the job and report errors
 * to the user, errors are deferred and logged in slurm_spank_task_init(), which
 * then returns SLURM_ERROR to trigger job cancellation.
 */
static bool g_init_post_opt_failed = false;

#ifndef PRIOR_TO_V24_05_5_1
/* List of acquired QPU resources. */
static list_t *g_acquired_resources = NULL;
/* List of errors deferred from slurm_spank_init_post_opt(). */
static list_t *g_init_post_opt_errors = NULL;
#else
/* List of acquired QPU resources. */
static List g_acquired_resources = NULL;
/* List of errors deferred from slurm_spank_init_post_opt(). */
static List g_init_post_opt_errors = NULL;
#endif /* !PRIOR_TO_V24_05_5_1 */

/*
 *  Forward declarations
 */
static qpu_resource_t *_acquired_resource_create(char *name, QrmiResourceType type,
                                                 const char *token);
static void acquired_resource_destroy(void *object);
static qpu_resource_t *_acquire_qpu(spank_t spank_ctxt, char *name, QrmiResourceType type);
static void _release_qpu(qpu_resource_t *res);
static qrmi_error_t *_qrmi_error_create(char* message);
static void qrmi_error_destroy(void *object);

/*
 * @function _dump_environ
 *
 * Dumps all environment variables set for the current process.
 */
static void _dump_environ(void) {
    char **s = environ;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): environment variables ---", plugin_name, pid, uid);
    for (; *s; s++) {
        slurm_debug("%s(%d, %d): %s", plugin_name, pid, uid, *s);
    }
}

/*
 * @function _starts_with
 *
 * Tests if this string(`str`) starts with the specified `prefix`.
 */
static bool _starts_with(const char *str, const char *prefix) {
    return strncmp(prefix, str, strlen(prefix)) == 0;
}

/*
 * @function _qpu_names_opt_cb
 *
 * A callback function that is invoked when the `--qpu` plugin option is registered with Slurm.
 */
static int _qpu_names_opt_cb(int val, const char *optarg, int remote) {
    UNUSED_PARAM(val);
    UNUSED_PARAM(remote);
    g_qpu_names_opt = strdup(optarg);
    if (g_qpu_names_opt == NULL) {
        slurm_error("%s, Failed to duplicate '--qpu' option value", plugin_name);
        return SLURM_ERROR;
    }
    slurm_debug("%s: --qpu=[%s]", plugin_name, g_qpu_names_opt);
    return SLURM_SUCCESS;
}

/*
 * Options available to this spank plugin.
 */
struct spank_option spank_qrmi_options[] = {
    {(char *)"qpu", (char *)"names", (char *)"Comma separated list of QPU resources to use.",
     1, /* argument is required */
     0, /* value to return using callback */
     (spank_opt_cb_f)_qpu_names_opt_cb},
    SPANK_OPTIONS_TABLE_END};

/*
 * @function slurm_qrmi_error
 *
 * Formats an error message using printf-style format and variable arguments,
 * appends it to the global error list (g_init_post_opt_errors), and reports it
 * via slurm_error().
 *
 * Note: Messages exceeding 4096 bytes will be silently truncated.
 */
__attribute__((format(printf, 1, 2)))
static void slurm_qrmi_error(const char *format, ...) {
    char buf[MAX_ERROR_STRLEN+1];
    va_list args;
    va_start(args, format);

    vsnprintf(buf, sizeof(buf), format, args);
    va_end(args);
    qrmi_error_t *qrmi_err = _qrmi_error_create(buf);
    if (qrmi_err) {
        slurm_list_append(g_init_post_opt_errors, qrmi_err);
    }
    slurm_error("%s", buf);
}

/*
 * @function slurm_spank_init
 *
 * Called just after plugins are loaded. In remote context, this is just after
 * job step is initialized. This function is called before any plugin option
 * processing.
 *
 */
int slurm_spank_init(spank_t spank_ctxt, int argc, char *argv[]) {
    UNUSED_PARAM(argv);
    struct spank_option *opts_to_register = NULL;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid, __func__, argc,
                spank_remote(spank_ctxt));

    g_acquired_resources = slurm_list_create(acquired_resource_destroy);
    g_init_post_opt_errors = slurm_list_create(qrmi_error_destroy);

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
        while (opts_to_register->name) {
            if (spank_option_register(spank_ctxt, opts_to_register++) != ESPANK_SUCCESS) {
                return SLURM_ERROR;
            }
        }
    }
    slurm_debug("%s(%d,%d): <- %s", plugin_name, pid, uid, __func__);

    return SLURM_SUCCESS;
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
    uint32_t job_stepid = 0;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid, __func__, argc,
                spank_remote(spank_ctxt));

    if (!spank_remote(spank_ctxt)) {
        /* never occurred. just for safe. */
        return SLURM_SUCCESS;
    }

    if (spank_get_item(spank_ctxt, S_JOB_STEPID, &job_stepid) == ESPANK_SUCCESS) {
        /* skip if this is slurm task steps */
        slurm_debug("%s, job_stepid = %x", plugin_name, job_stepid);
        if (job_stepid != SLURM_BATCH_SCRIPT) {
            return SLURM_SUCCESS;
        }
    }

    if (g_qpu_names_opt == NULL) {
        /* noop if this is not QPU job */
        return SLURM_SUCCESS;
    }

    /*
     * Set environment variable for slurm job ID and UID.
     */
    uid_t job_uid;
    if (spank_get_item(spank_ctxt, S_JOB_UID, &job_uid) != ESPANK_SUCCESS) {
        slurm_qrmi_error("%s, unable to get job UID", plugin_name);
        g_init_post_opt_failed = true;
        return SLURM_SUCCESS;
    }
    char uid_str[16];
    snprintf(uid_str, sizeof(uid_str), "%u", job_uid);
    spank_setenv(spank_ctxt, "QRMI_JOB_UID", uid_str, OVERWRITE);
    setenv("QRMI_JOB_UID", uid_str, OVERWRITE);

    uint32_t job_id;
    if (spank_get_item(spank_ctxt, S_JOB_ID, &job_id) != ESPANK_SUCCESS) {
        slurm_qrmi_error("%s, unable to get job ID", plugin_name);
        g_init_post_opt_failed = true;
        return SLURM_SUCCESS;
    }
    char id_str[12];
    snprintf(id_str, sizeof(id_str), "%u", job_id);
    spank_setenv(spank_ctxt, "QRMI_JOB_ID", id_str, OVERWRITE);
    setenv("QRMI_JOB_ID", id_str, OVERWRITE);

    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_RESOURCES", "", OVERWRITE);
    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_TYPES", "", OVERWRITE);

    for (int i = 0; i < argc; i++) {
        slurm_debug("%s: argv[%d] = [%s]", plugin_name, i, argv[i]);
    }

    if (argc == 0) {
        slurm_qrmi_error("%s, QRMI config file not specified to plugin args", plugin_name);
        g_init_post_opt_failed = true;
        return SLURM_SUCCESS;
    }

    QrmiConfig *cnf = qrmi_config_load(argv[0]);
    if (cnf == NULL) {
        const char *last_error = qrmi_get_last_error();
        slurm_qrmi_error("%s, Failed to load QRMI config file(%s). %s", plugin_name, argv[0],
                    last_error);
        g_init_post_opt_failed = true;
        return SLURM_SUCCESS;
    }
    slurm_debug("%s, config: %p", plugin_name, (void *)cnf);

    /*
     * Parses optional plugin arguments.
     *
     * Currently, only environment variable settings prefixed with
     * --env:{variable name}={value} are supported.
     */
    for (int i = 1; i < argc; i++) {
        if (!_starts_with(argv[i], "--env:")) {
            /* ignored. */
            continue;
        }
        const char *input = &argv[i][strlen("--env:")];
        const char *delimiter = strchr(input, '=');
        if (delimiter == NULL) {
            slurm_qrmi_error("%s, Invalid --env: argument. '=' delimiter not found in %s", plugin_name, argv[i]);
            g_init_post_opt_failed = true;
            return SLURM_SUCCESS;
        }
        size_t env_name_len = (size_t)(delimiter - input);
        char *env_name = strndup(input, env_name_len);
        if (env_name == NULL) {
            slurm_qrmi_error("%s, Failed to allocate buffer with length = %ld", plugin_name, env_name_len);
            g_init_post_opt_failed = true;
            return SLURM_SUCCESS;
        }
        const char *env_value = delimiter + 1;
        setenv(env_name, env_value, OVERWRITE);
        spank_setenv(spank_ctxt, env_name, env_value, OVERWRITE);
        free(env_name);
    }

    char *bufp = strdup(g_qpu_names_opt);
    if (bufp == NULL) {
        slurm_qrmi_error("%s, Failed to allocate buffer with length = %ld", plugin_name, strlen(g_qpu_names_opt));
        g_init_post_opt_failed = true;
        return SLURM_SUCCESS;
    }
    char *rest = bufp;
    char *token;
    buffer keybuf;
    qrmi_buf_init(&keybuf, 1024);

    while ((token = strtok_r(rest, ",", &rest))) {
        QrmiResourceDef *res = qrmi_config_resource_def_get(cnf, token);
        if (res != NULL) {
            slurm_debug("%s: name(%s), type(%d) found in %s", plugin_name, res->name, res->type,
                        argv[0]);
            /*
             * If user specifies access details in environment variables,
             * these are available as job environment variables. Reads through
             * them and set user-specified {qpu_name}_QRMI_xxx env vars to this
             * slurm daemon process for subsequent QRMI.acquire/release call.
             */
            qrmi_buf_envvarname_for_res_create(&keybuf, res->name, "QRMI_");
            char **job_argv = NULL;
            if (spank_get_item(spank_ctxt, S_JOB_ENV, &job_argv) == ESPANK_SUCCESS) {
                int index = 0;
                while (job_argv[index] != NULL) {
                    if (strncmp(job_argv[index], keybuf.buffer, strlen(keybuf.buffer)) == 0) {
                        const char *kv_text = job_argv[index];
                        const char *eq = strchr(kv_text, '=');
                        if (eq) {
                            char *key = strndup(kv_text, (size_t)(eq - kv_text));
                            if (key == NULL) {
                                slurm_qrmi_error("%s, Failed to allocate buffer with length = %ld", plugin_name, eq - kv_text);
                                g_init_post_opt_failed = true;
                                free(bufp);
                                qrmi_buf_free(&keybuf);
                                return SLURM_SUCCESS;
                            }
                            const char *value = eq + 1;
                            slurm_debug("%s: putenv(%s, %s)", plugin_name, key, value);
                            setenv(key, value, OVERWRITE);
                            free(key);
                        }
                    }
                    index++;
                }
            }

            /*
             * Next, set environment variables specified in config file.
             */
            QrmiEnvironmentVariables envvars = res->environments;
            for (size_t j = 0; j < envvars.length; j++) {
                QrmiKeyValue envvar = envvars.variables[j];
                /* set to the current process for subsequent QRMI.acquire() call
                 */
                qrmi_buf_envvarname_for_res_create(&keybuf, res->name, envvar.key);
                slurm_debug("%s: setenv(%s, %s)", plugin_name, keybuf.buffer, envvar.value);
                setenv(keybuf.buffer, envvar.value, KEEP_IF_EXISTS);
                spank_setenv(spank_ctxt, keybuf.buffer, envvar.value, KEEP_IF_EXISTS);
            }

            _dump_environ();

            /*
             * Acquire QPU resource.
             */
            qpu_resource_t *acquired = _acquire_qpu(spank_ctxt, res->name, res->type);
            if (acquired != NULL) {
                slurm_list_append(g_acquired_resources, acquired);
                qrmi_buf_envvarname_for_res_create(&keybuf, res->name,
                                                   "QRMI_JOB_ACQUISITION_TOKEN");
                slurm_debug("%s: setenv(%s, %s)", plugin_name, keybuf.buffer,
                            acquired->acquisition_token);
                setenv(keybuf.buffer, acquired->acquisition_token, KEEP_IF_EXISTS);
                spank_setenv(spank_ctxt, keybuf.buffer, acquired->acquisition_token,
                             KEEP_IF_EXISTS);
            } else {
                slurm_qrmi_error("%s, failed to acquire resource: %s", plugin_name, res->name);
            }
            qrmi_config_resource_def_free(res);
        } else {
            slurm_qrmi_error("resource %s not found in %s", token, argv[0]);
        }
    }
    free(bufp);
    qrmi_buf_free(&keybuf);

    if (slurm_list_count(g_acquired_resources) == 0) {
        slurm_qrmi_error("%s, No QPU resource available", plugin_name);
        g_init_post_opt_failed = true;
        return SLURM_SUCCESS;
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
        slurm_debug("%s: name(%s), type(%d), token(%s)", plugin_name, item->name, item->type,
                    item->acquisition_token);
        strbuf_append_str(&qpu_resources_envvar, item->name);
        const char *type_as_str = qrmi_config_resource_type_to_str(item->type);
        slurm_debug("%s: type_as_str(%s)", plugin_name, type_as_str);
        strbuf_append_str(&qpu_types_envvar, qrmi_config_resource_type_to_str(item->type));
        qrmi_string_free((char *)type_as_str);
    }
    slurm_list_iterator_destroy(sessions_iter);

    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_RESOURCES", qpu_resources_envvar.buffer, OVERWRITE);
    slurm_debug("%s: setenv(%s, %s)", plugin_name, "SLURM_JOB_QPU_RESOURCES",
                qpu_resources_envvar.buffer);
    spank_setenv(spank_ctxt, "SLURM_JOB_QPU_TYPES", qpu_types_envvar.buffer, OVERWRITE);
    slurm_debug("%s: setenv(%s, %s)", plugin_name, "SLURM_JOB_QPU_TYPES", qpu_types_envvar.buffer);
    strbuf_free(&qpu_resources_envvar);
    strbuf_free(&qpu_types_envvar);

    slurm_debug("%s(%d,%d): <- %s", plugin_name, pid, uid, __func__);
    return SLURM_SUCCESS;
}

/*
 * @function slurm_spank_task_init
 *
 * Called for each task just before execve (2). If you are restricting memory
 * with cgroups, memory allocated here will be in the job's cgroup. (remote
 * context only)
 */
int slurm_spank_task_init(spank_t spank_ctxt, int argc, char **argv) {
    UNUSED_PARAM(argv);
    uint32_t job_id = 0;
    job_info_msg_t *job_info_msg = NULL;
    int pid = (int)getpid();
    int uid = (int)getuid();

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid, __func__, argc,
                spank_remote(spank_ctxt));

    if (!spank_remote(spank_ctxt)) {
        /* never occurred. just for safe. */
        return SLURM_SUCCESS;
    }

    if (g_init_post_opt_failed == true) {
        /* g_init_post_opt_failed is set, meaning an error occurred in
         * slurm_spank_init_post_opt(). Log the deferred errors and return
         * SLURM_ERROR to cancel the job and notify the user of the failure reason.
         */
#ifndef PRIOR_TO_V24_05_5_1
        list_itr_t *errors_iter =
#else
        ListIterator errors_iter =
#endif /* !PRIOR_TO_V24_05_5_1 */
            slurm_list_iterator_create(g_init_post_opt_errors);
        void *e = NULL;
        while ((e = slurm_list_next(errors_iter)) != NULL) {
            qrmi_error_t *item = (qrmi_error_t *)e;
            slurm_error("%s", item->message);
        }
        slurm_list_iterator_destroy(errors_iter);
        return SLURM_ERROR;
    }

    char *optargp = NULL;
    if (spank_option_getopt(spank_ctxt, &spank_qrmi_options[0], &optargp) != ESPANK_SUCCESS) {
        /* noop if this is not QPU job */
        return SLURM_SUCCESS;
    }
    if (optargp == NULL || strlen(optargp) == 0) {
        /* noop if this is not QPU job */
        return SLURM_SUCCESS;
    }

    char limit_as_str[MAX_INT_STRLEN + 1] = {0}; /* max uint32_t value is (2147483647) = 10 chars */
    if (spank_get_item(spank_ctxt, S_JOB_ID, &job_id) == ESPANK_SUCCESS) {
        if (slurm_load_job(&job_info_msg, job_id, SHOW_DETAIL) == SLURM_SUCCESS) {
            /*
             * Slurm's time limit is represented in minutes
             */
            uint32_t time_limit_mins = job_info_msg->job_array[0].time_limit;
            /*
             * Convert minutes to seconds, uint32_t to char*
             */
            if (time_limit_mins != INFINITE && time_limit_mins <= (UINT32_MAX / 60)) {
                snprintf(limit_as_str, sizeof(limit_as_str), "%u", time_limit_mins * 60);
            } else {
                /* time limit too large or INFINITE; clamp to max or handle as error */
                snprintf(limit_as_str, sizeof(limit_as_str), "%u", UINT32_MAX);
            }
        }
    }

    if (strlen(limit_as_str) == 0) {
        /* time limit should be there, something wrong in Slurm */
        return SLURM_ERROR;
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
        qrmi_buf_envvarname_for_res_create(&keybuf, res->name, "QRMI_JOB_TIMEOUT_SECONDS");
        spank_setenv(spank_ctxt, keybuf.buffer, limit_as_str, OVERWRITE);
        slurm_debug("%s: setenv(%s, %s)", plugin_name, keybuf.buffer, limit_as_str);
    }
    slurm_list_iterator_destroy(resources_iter);
    qrmi_buf_free(&keybuf);

    /*
     * Set environment variables for QRMI runtime logging.
     */
    buffer srun_debug;
    qrmi_buf_init(&srun_debug, MAX_INT_STRLEN + 1);
    if (spank_getenv(spank_ctxt, "SRUN_DEBUG", srun_debug.buffer, MAX_INT_STRLEN + 1) ==
        ESPANK_SUCCESS) {
        /* if failed, level=0 --> default level(info) */
        int level = atoi(srun_debug.buffer);
        const char *level_str = NULL;
        switch (level) {
        case 2:
            /* --quiet */
            level_str = "error";
            break;
        case 3:
            /* default */
            level_str = "info";
            break;
        case 4:
            /* --verbose */
            level_str = "debug";
            break;
        default:
            if (level >= 5) {
                /* -vv or more */
                level_str = "trace";
            } else {
                /* default is Info as same as srun */
                level_str = "info";
            }
            break;
        }
        if (level_str != NULL) {
            spank_setenv(spank_ctxt, "RUST_LOG", level_str, KEEP_IF_EXISTS);
            slurm_debug("%s: setenv(%s, %s)", plugin_name, "RUST_LOG", level_str);
        }
    }
    qrmi_buf_free(&srun_debug);

    slurm_debug("%s(%d,%d): <- %s", plugin_name, pid, uid, __func__);

    return SLURM_SUCCESS;
}

/*
 * @function slurm_spank_exit
 *
 * Called once just before slurmstepd exits in remote context. In local
 * context, called before srun exits.
 */
int slurm_spank_exit(spank_t spank_ctxt, int argc, char **argv) {
    UNUSED_PARAM(argv);
    int pid = (int)getpid();
    int uid = (int)getuid();

    if (!spank_remote(spank_ctxt)) {
        /* never occurred. just for safe. */
        return SLURM_SUCCESS;
    }

    slurm_debug("%s(%d, %d): -> %s argc=%d remote=%d", plugin_name, pid, uid, __func__, argc,
                spank_remote(spank_ctxt));

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

    slurm_list_destroy(g_init_post_opt_errors);
    g_init_post_opt_errors = NULL;

    if (g_qpu_names_opt != NULL) {
        free(g_qpu_names_opt);
        g_qpu_names_opt = NULL;
    }

    slurm_debug("%s(%d,%d): <- %s", plugin_name, pid, uid, __func__);

    return SLURM_SUCCESS;
}

/*
 * @function _acquired_resource_create
 *
 * Constructs an acquired QPU resource record. See
 * acquired_resource_destroy() to free allocated memory.
 */
static qpu_resource_t *_acquired_resource_create(char *name, QrmiResourceType type,
                                                 const char *token) {
    /* Copies name, type, and token into a newly allocated qpu_resource_t. */
    qpu_resource_t *info = malloc(sizeof(qpu_resource_t));
    if (info == NULL) {
        return NULL;
    }

    info->name = strdup(name);
    if (info->name == NULL) {
        free(info);
        return NULL;
    }

    info->type = type;

    info->acquisition_token = strdup(token);
    if (info->acquisition_token == NULL) {
        free(info->name);
        free(info);
        return NULL;
    }
    return info;
}

/*
 * @function _qrmi_error_create
 *
 * Constructs a record of error occurred in init_post_opt(). See
 * qrmi_error_destroy() to free allocated memory.
 */
static qrmi_error_t *_qrmi_error_create(char *message) {
    /* Copies message into a newly allocated qrmi_error_t. */
    qrmi_error_t *err = malloc(sizeof(qrmi_error_t));
    if (err == NULL) {
        return NULL;
    }

    err->message = strdup(message);
    if (err->message == NULL) {
        free(err);
        return NULL;
    }
    return err;
}

/*
 * @function acquired_resource_destroy
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
 * @function qrmi_error_destroy
 *
 * Destroy a QRMI error record.
 */
static void qrmi_error_destroy(void *object) {
    qrmi_error_t *err = (qrmi_error_t *)object;

    free(err->message);
    free(err);
}

/*
 * @function _acquire_qpu
 *
 * Acquire QPU resource specified by `name` and `type`. Returns
 * qpu_resource_t object if succeeded.
 */
static qpu_resource_t *_acquire_qpu(spank_t spank_ctxt, char *name, QrmiResourceType type) {
    char *acquisition_token = NULL;
    bool is_accessible = false;
    QrmiReturnCode rc;
    const char *last_error = NULL;
    UNUSED_PARAM(spank_ctxt);

    void *qrmi = qrmi_resource_new(name, type);
    if (qrmi == NULL) {
        last_error = qrmi_get_last_error();
        slurm_qrmi_error("%s, %s", plugin_name, last_error);
        return NULL;
    }

    slurm_debug("%s, qrmi: %p", plugin_name, qrmi);
    rc = qrmi_resource_is_accessible(qrmi, &is_accessible);
    if ((rc != QRMI_RETURN_CODE_SUCCESS) || (is_accessible == false)) {
        last_error = qrmi_get_last_error();
        slurm_qrmi_error("%s, %s is not accessible. %s", plugin_name, name, last_error);
        qrmi_resource_free(qrmi);
        return NULL;
    }
    rc = qrmi_resource_acquire(qrmi, &acquisition_token);
    qrmi_resource_free(qrmi);
    if ((rc != QRMI_RETURN_CODE_SUCCESS) || (acquisition_token == NULL)) {
        last_error = qrmi_get_last_error();
        slurm_qrmi_error("%s, resource acquisition failed: %s. %s", plugin_name, name, last_error);
        return NULL;
    }

    slurm_debug("%s, acquisition_token: %s(%s)", plugin_name, acquisition_token, name);
    qpu_resource_t *res =_acquired_resource_create(name, type, acquisition_token);
    qrmi_string_free(acquisition_token);
    return res;
}

/*
 * @function _release_qpu
 *
 * Release QPU resource which was acquired by _acquired_qpu().
 *
 * This function is called in the exit phase, so deferring errors to the list via
 * slurm_qrmi_error() is not needed here - use slurm_error() as usual.
 */
static void _release_qpu(qpu_resource_t *res) {
    QrmiReturnCode rc;

    if (res == NULL) {
        return;
    }
    slurm_debug("%s: releasing name(%s), type(%d), token(%s)", plugin_name, res->name, res->type,
                res->acquisition_token);
    void *qrmi = qrmi_resource_new(res->name, res->type);
    if (qrmi == NULL) {
        const char *last_error = qrmi_get_last_error();
        slurm_error("%s, %s", plugin_name, last_error);
        return;
    }
    rc = qrmi_resource_release(qrmi, res->acquisition_token);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
        const char *last_error = qrmi_get_last_error();
        slurm_error("%s, Failed to release acquired resource: name(%s), type(%d), token(%s), %s",
                    plugin_name, res->name, res->type, res->acquisition_token, last_error);
    }
    rc = qrmi_string_free(res->acquisition_token);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
        slurm_error("%s, Failed to free acquisition token string: (%s)", plugin_name,
                    res->acquisition_token);
    }
    rc = qrmi_resource_free(qrmi);
    if (rc != QRMI_RETURN_CODE_SUCCESS) {
        slurm_error("%s, Failed to free QrmiQuantumResource handle: (%p)", plugin_name, qrmi);
    }
    res->acquisition_token = NULL;
}
