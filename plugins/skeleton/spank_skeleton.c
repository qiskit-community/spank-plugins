/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM 2025.
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
 * along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
 */
#include <ctype.h>
#include <grp.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <slurm/slurm.h>
#include <slurm/spank.h>

/*
 * All spank plugins must define this macro for the SLURM plugin loader.
 */
SPANK_PLUGIN(spank_skeleton, 1)

static const int PLUGIN_ARGC = 7;

#define MAXLEN_BUF 256
static char buf[MAXLEN_BUF + 1];

/*
 * @function strncpy_s
 *
 * For security. Copy string src to buffer dst of size dsize.  At most dsize-1
 * chars will be copied. Always NULL terminates (unless dsize == 0).
 */
static char *strncpy_s(char *dst, const char *src, size_t dsize)
{
    if (dsize == 0) {
        return dst;
    }

    strncpy(dst, src, dsize - 1);
    dst[dsize - 1] = '\0';
    return dst;
}

/*
 * @function backend_name_cb
 *
 * Callback function to check --q-backend option value.
 *
 */
static int _callback(int val, const char *optarg, int remote)
{
    slurm_debug("%s: %s val=%d optarg=%s remote=%d",
            plugin_name, __FUNCTION__, val, optarg, remote);
    strncpy_s(buf, optarg, sizeof(buf));
    return ESPANK_SUCCESS;
}

static int dump_spank_items(spank_t spank_ctxt)
{
    uid_t job_id = -1;
    uint32_t step_id = -1;
    int job_argc = 0;
    char **job_argv = NULL;

    if (spank_get_item(spank_ctxt, S_JOB_UID, &job_id) == ESPANK_SUCCESS) {
        slurm_debug("%s: S_JOB_UID [%d]", plugin_name, job_id);
    }
    if (spank_get_item(spank_ctxt, S_JOB_ID, &step_id) == ESPANK_SUCCESS) {
        slurm_debug("%s: S_JOB_ID [%d]", plugin_name, step_id);
    }

    if (spank_get_item(spank_ctxt, S_JOB_ARGV, &job_argc, &job_argv) ==
        ESPANK_SUCCESS) {
        slurm_debug("%s: S_JOB_ARGV argc=%d", plugin_name, job_argc);
        for (int i = 0; i < job_argc; i++) {
            slurm_debug("%s: job_argv[%d] = [%s]",
                    plugin_name, i, job_argv[i]);
        }
    }
    return ESPANK_SUCCESS;
}

static int dump_argv(int argc, char **argv)
{
    for (int i = 0; i < argc; i++) {
        slurm_debug("%s: argv[%d] = [%s]", plugin_name, i, argv[i]);
    }
    return ESPANK_SUCCESS;
}

/*
 * Options available to this spank plugin:
 */
struct spank_option spank_example_options[] = {
    {
        "skeleton-option",
        "value",
        "Option for spank-skeleton.",
        1, /* argument is required */
        0, /* value to return using callback */
        (spank_opt_cb_f)_callback
    },
    SPANK_OPTIONS_TABLE_END
};

/*
 * @function slurm_spank_init
 *
 * Called just after plugins are loaded. In remote context, this is just after
 * job step is initialized. This function is called before any plugin option
 * processing.
 *
 */
int slurm_spank_init(spank_t spank_ctxt, int argc, char *argv[])
{
    int rc = ESPANK_SUCCESS;
    struct spank_option *opts_to_register = NULL;

    slurm_debug("%s: -> %s argc=%d", plugin_name, __FUNCTION__, argc);
    dump_argv(argc, argv);

    memset(buf, '\0', sizeof(buf));
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
        opts_to_register = spank_example_options;
        break;

    default:
        break;
    }
    if (opts_to_register) {
        while (opts_to_register->name && (rc == ESPANK_SUCCESS)) {
            rc = spank_option_register(spank_ctxt, opts_to_register++);
        }
    }

    /*
     * SPANK plugins can query the current list of supported slurm_spank symbols
     * to determine if the current version supports a given plugin hook.
     * This may be useful because the list of plugin symbols may grow in the
     * future.
     */
    slurm_debug("%s Is slurm_spank_task_init() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_init"));
    slurm_debug("%s Is slurm_spank_job_prolog() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_job_prolog"));
    slurm_debug("%s Is slurm_spank_init_post_opt() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_init_post_opt"));
    slurm_debug("%s Is slurm_spank_local_user_init() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_local_user_init"));
    slurm_debug("%s Is slurm_spank_user_init() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_user_init"));
    slurm_debug("%s Is slurm_spank_task_init_privileged() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_init_privileged"));
    slurm_debug("%s Is slurm_spank_task_init() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_init"));
    slurm_debug("%s Is slurm_spank_task_post_fork() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_post_fork"));
    slurm_debug("%s Is slurm_spank_task_exit() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_task_exit"));
    slurm_debug("%s Is slurm_spank_exit() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_exit"));
    slurm_debug("%s Is slurm_spank_job_epilog() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_job_epilog"));
    slurm_debug("%s Is slurm_spank_slurmd_exit() supported ? %d", plugin_name,
        spank_symbol_supported("slurm_spank_slurmd_exit"));

    slurm_debug("%s <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}


/*
 * @function slurm_spank_job_prolog
 *
 * Called at the same time as the job prolog. If this function returns a non-zero value
 * and the SPANK plugin that contains it is required in the plugstack.conf, the node that
 * this is run on will be drained.
 *
 */
int slurm_spank_job_prolog(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_init_post_opt
 *
 * Called at the same point as slurm_spank_init, but after all user options to the plugin
 * have been processed. The reason that the init and init_post_opt callbacks are separated
 * is so that plugins can process system-wide options specified in plugstack.conf in the
 * init callback, then process user options, and finally take some action in
 * slurm_spank_init_post_opt if necessary. In the case of a heterogeneous job, slurm_spank_init
 * is invoked once per job component.
 *
 */
int slurm_spank_init_post_opt(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_local_user_init
 *
 * Called in local (srun) context only after all options have been processed. This is called
 * after the job ID and step IDs are available. This happens in srun after the allocation is made,
 * but before tasks are launched.
 *
 */
int slurm_spank_local_user_init(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_user_init
 *
 * Called after privileges are temporarily dropped. (remote context only)
 *
 */
int slurm_spank_user_init(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_task_init_privileged
 *
 * Called for each task just after fork, but before all elevated privileges are dropped.
 * (remote context only)
 *
 */
int slurm_spank_task_init_privileged(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_task_init
 *
 * Called for each task just before execve (2). If you are restricting memory
 * with cgroups, memory allocated here will be in the job's cgroup. (remote
 * context only)
 *
 */
int slurm_spank_task_init(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;
    int i;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);
    dump_spank_items(spank_ctxt);

    if (spank_remote(spank_ctxt)) {
        if (strlen(buf) > 0) {
            slurm_debug("%s: setenv SPANK_SKELETON=%s",
                plugin_name, buf);
            spank_setenv(spank_ctxt, "SPANK_SKELETON", buf, 1);
        }
    }

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_task_post_fork
 *
 * Called for each task from parent process after fork (2) is complete. Due to the fact that 
 * slurmd does not exec any tasks until all tasks have completed fork (2), this call is 
 * guaranteed to run before the user task is executed. (remote context only)
 *
 */
int slurm_spank_task_post_fork(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_task_exit
 *
 * Called for each task as its exit status is collected by Slurm. (remote
 * context only)
 */
int slurm_spank_task_exit(spank_t spank_ctxt, int argc, char **argv) 
{
    int rc = ESPANK_SUCCESS;
    int status;

    slurm_debug("%s: -> %s argc=%d", plugin_name, __FUNCTION__, argc);
    dump_argv(argc, argv);

    if (spank_get_item(spank_ctxt, S_TASK_EXIT_STATUS, &status) ==
        ESPANK_SUCCESS) {
        slurm_debug("%s: S_TASK_EXIT_STATUS [%d]", plugin_name, status);
    }

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_exit
 *
 * Called once just before slurmstepd exits in remote context. In local context,
 * called before srun exits.
 *
 */
int slurm_spank_exit(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_job_epilog
 *
 * Called at the same time as the job epilog. If this function returns a non-zero value and
 * the SPANK plugin that contains it is required in the plugstack.conf, the node that this is
 * run on will be drained.
 *
 */
int slurm_spank_job_epilog(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}

/*
 * @function slurm_spank_slurmd_exit
 *
 * Called at the same time as the job epilog. If this function returns a non-zero value and
 * the SPANK plugin that contains it is required in the plugstack.conf, the node that this is
 * run on will be drained.
 *
 */
int slurm_spank_slurmd_exit(spank_t spank_ctxt, int argc, char **argv)
{
    int rc = ESPANK_SUCCESS;

    slurm_debug("%s: -> %s argc=%d remote=%d", plugin_name, __FUNCTION__, argc,
            spank_remote(spank_ctxt));
    dump_argv(argc, argv);

    slurm_debug("%s: <- %s rc=%d", plugin_name, __FUNCTION__, rc);
    return rc;
}
