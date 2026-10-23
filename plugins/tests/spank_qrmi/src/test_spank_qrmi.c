/*
 * This code is part of Qiskit.
 *
 * (C) Copyright Pasqal 2026
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

/*
 * Behavioural tests for spank_qrmi. The plugin source is included so its static
 * functions and globals are visible; the Slurm and QRMI functions it calls are
 * replaced by the fakes below. Run one case per process: `test_spank_qrmi <case>`.
 */
#include "spank_qrmi.c"

#include <signal.h>
#include <stdio.h>

/* ---------------------------------------------------------------- fake Slurm */

#define MAX_ENV 64
#define LOG_SIZE 8192

static struct {
    char *env[MAX_ENV + 1]; /* job environment as "KEY=VALUE", NULL terminated */
    uint32_t stepid;
    uint32_t time_limit_mins;
    char errors[LOG_SIZE]; /* every slurm_error() message, one per line */
} g_job = {.stepid = SLURM_BATCH_SCRIPT, .time_limit_mins = 10};

static void _log_error(const char *format, va_list args) {
    size_t len = strlen(g_job.errors);
    vsnprintf(g_job.errors + len, sizeof(g_job.errors) - len, format, args);
    strncat(g_job.errors, "\n", sizeof(g_job.errors) - strlen(g_job.errors) - 1);
}

void slurm_error(const char *format, ...) {
    va_list args;
    va_start(args, format);
    _log_error(format, args);
    va_end(args);
}

void slurm_info(const char *format, ...) { UNUSED_PARAM(format); }
void slurm_debug(const char *format, ...) { UNUSED_PARAM(format); }
void slurm_debug2(const char *format, ...) { UNUSED_PARAM(format); }

static char **_job_env_find(const char *key) {
    size_t key_len = strlen(key);
    for (char **e = g_job.env; *e; e++) {
        if (strncmp(*e, key, key_len) == 0 && (*e)[key_len] == '=') {
            return e;
        }
    }
    return NULL;
}

static const char *job_env_get(const char *key) {
    char **e = _job_env_find(key);
    return e ? strchr(*e, '=') + 1 : NULL;
}

static void job_env_add(const char *entry) {
    size_t n = 0;
    while (g_job.env[n]) {
        n++;
    }
    g_job.env[n] = strdup(entry);
}

int spank_remote(spank_t spank) {
    UNUSED_PARAM(spank);
    return 1;
}

spank_context_t spank_context(void) { return S_CTX_REMOTE; }

spank_err_t spank_option_register(spank_t spank, struct spank_option *opt) {
    UNUSED_PARAM(spank);
    UNUSED_PARAM(opt);
    return ESPANK_SUCCESS;
}

spank_err_t spank_option_getopt(spank_t spank, struct spank_option *opt, char **optarg) {
    UNUSED_PARAM(spank);
    UNUSED_PARAM(opt);
    if (g_qpu_names_opt == NULL) {
        return ESPANK_ERROR;
    }
    *optarg = g_qpu_names_opt;
    return ESPANK_SUCCESS;
}

spank_err_t spank_get_item(spank_t spank, spank_item_t item, ...) {
    UNUSED_PARAM(spank);
    spank_err_t rc = ESPANK_SUCCESS;
    va_list args;
    va_start(args, item);
    switch (item) {
    case S_JOB_UID:
        *va_arg(args, uid_t *) = 1000;
        break;
    case S_JOB_ID:
        *va_arg(args, uint32_t *) = 42;
        break;
    case S_JOB_STEPID:
        *va_arg(args, uint32_t *) = g_job.stepid;
        break;
    case S_JOB_ENV:
        *va_arg(args, char ***) = g_job.env;
        break;
    default:
        rc = ESPANK_BAD_ARG;
    }
    va_end(args);
    return rc;
}

spank_err_t spank_getenv(spank_t spank, const char *var, char *buf, int len) {
    UNUSED_PARAM(spank);
    const char *value = job_env_get(var);
    if (value == NULL) {
        return ESPANK_ENV_NOEXIST;
    }
    snprintf(buf, (size_t)len, "%s", value);
    return ESPANK_SUCCESS;
}

spank_err_t spank_setenv(spank_t spank, const char *var, const char *val, int overwrite) {
    UNUSED_PARAM(spank);
    char **e = _job_env_find(var);
    if (e != NULL && !overwrite) {
        return ESPANK_ENV_EXISTS;
    }
    char entry[1024];
    snprintf(entry, sizeof(entry), "%s=%s", var, val);
    if (e != NULL) {
        free(*e);
        *e = strdup(entry);
    } else {
        job_env_add(entry);
    }
    return ESPANK_SUCCESS;
}

/* A minimal list: slurm_list_* only needs append, count and iteration. */
struct xlist {
    void *items[16];
    int count;
    ListDelF destroy;
};

struct listIterator {
    list_t *list;
    int pos;
};

list_t *slurm_list_create(ListDelF f) {
    list_t *l = calloc(1, sizeof(*l));
    l->destroy = f;
    return l;
}

void slurm_list_append(list_t *l, void *x) { l->items[l->count++] = x; }

int slurm_list_count(list_t *l) { return l->count; }

void slurm_list_destroy(list_t *l) {
    for (int i = 0; i < l->count; i++) {
        l->destroy(l->items[i]);
    }
    free(l);
}

list_itr_t *slurm_list_iterator_create(list_t *l) {
    list_itr_t *i = calloc(1, sizeof(*i));
    i->list = l;
    return i;
}

void *slurm_list_next(list_itr_t *i) {
    return i->pos < i->list->count ? i->list->items[i->pos++] : NULL;
}

void slurm_list_iterator_destroy(list_itr_t *i) { free(i); }

static slurm_job_info_t g_job_info;
static job_info_msg_t g_job_info_msg = {.record_count = 1, .job_array = &g_job_info};

#if SLURM_VERSION_NUMBER >= SLURM_VERSION_NUM(26, 5, 0)
int slurm_load_job(job_info_msg_t **resp, slurm_step_id_t step_id, uint16_t show_flags) {
    UNUSED_PARAM(step_id);
#else
int slurm_load_job(job_info_msg_t **resp, uint32_t job_id, uint16_t show_flags) {
    UNUSED_PARAM(job_id);
#endif
    UNUSED_PARAM(show_flags);
    g_job_info.time_limit = g_job.time_limit_mins;
    *resp = &g_job_info_msg;
    return SLURM_SUCCESS;
}

void slurm_free_job_info_msg(job_info_msg_t *job_buffer_ptr) { UNUSED_PARAM(job_buffer_ptr); }

/* ----------------------------------------------------------------- fake QRMI */

typedef struct {
    const char *name;
    QrmiResourceType type;
    const char *type_str;
    bool accessible;
    bool acquire_ok;
    int acquired;
    char released_token[64];
} fake_resource_t;

static fake_resource_t g_resources[] = {
    {"local_qpu", QRMI_RESOURCE_TYPE_PASQAL_LOCAL, "pasqal-local", true, true, 0, ""},
    {"cloud_qpu", QRMI_RESOURCE_TYPE_PASQAL_CLOUD, "pasqal-cloud", true, true, 0, ""},
};
#define NUM_RESOURCES (sizeof(g_resources) / sizeof(g_resources[0]))

/* The one variable qrmi_config.json sets for every resource, as {name}_QRMI_URL. */
static QrmiKeyValue g_config_env = {(char *)"QRMI_URL", (char *)"http://config"};

static fake_resource_t *_resource(const char *name) {
    for (size_t i = 0; i < NUM_RESOURCES; i++) {
        if (strcmp(g_resources[i].name, name) == 0) {
            return &g_resources[i];
        }
    }
    return NULL;
}

/* Strings handed out by QRMI, so qrmi_string_free() can tell them from foreign ones. */
static char *g_qrmi_strings[64];
static int g_foreign_string_frees = 0;

static char *_qrmi_strdup(const char *s) {
    for (size_t i = 0; i < sizeof(g_qrmi_strings) / sizeof(g_qrmi_strings[0]); i++) {
        if (g_qrmi_strings[i] == NULL) {
            return g_qrmi_strings[i] = strdup(s);
        }
    }
    abort();
}

QrmiReturnCode qrmi_string_free(char *ptr) {
    for (size_t i = 0; i < sizeof(g_qrmi_strings) / sizeof(g_qrmi_strings[0]); i++) {
        if (ptr != NULL && g_qrmi_strings[i] == ptr) {
            free(ptr);
            g_qrmi_strings[i] = NULL;
            return QRMI_RETURN_CODE_SUCCESS;
        }
    }
    g_foreign_string_frees++;
    return QRMI_RETURN_CODE_ERROR;
}

char *qrmi_get_last_error(void) { return _qrmi_strdup("fake error"); }

#if defined(QRMI_HAS_LOG_CALLBACK)
QrmiReturnCode qrmi_log_callback_set(QrmiLogCallback callback) {
    UNUSED_PARAM(callback);
    return QRMI_RETURN_CODE_SUCCESS;
}
#endif

static bool g_config_load_fails = false;
static int g_config;

QrmiConfig *qrmi_config_load(const char *filename) {
    UNUSED_PARAM(filename);
    return g_config_load_fails ? NULL : (QrmiConfig *)&g_config;
}

QrmiReturnCode qrmi_config_free(QrmiConfig *ptr) {
    UNUSED_PARAM(ptr);
    return QRMI_RETURN_CODE_SUCCESS;
}

QrmiResourceDef *qrmi_config_resource_def_get(QrmiConfig *config, const char *resource_id) {
    UNUSED_PARAM(config);
    fake_resource_t *fake = _resource(resource_id);
    if (fake == NULL) {
        return NULL;
    }
    QrmiResourceDef *def = calloc(1, sizeof(*def));
    def->name = strdup(fake->name);
    def->type = fake->type;
    def->environments.variables = &g_config_env;
    def->environments.length = 1;
    return def;
}

QrmiReturnCode qrmi_config_resource_def_free(QrmiResourceDef *ptr) {
    free(ptr->name);
    free(ptr);
    return QRMI_RETURN_CODE_SUCCESS;
}

const char *qrmi_config_resource_type_to_str(QrmiResourceType type) {
    for (size_t i = 0; i < NUM_RESOURCES; i++) {
        if (g_resources[i].type == type) {
            return _qrmi_strdup(g_resources[i].type_str);
        }
    }
    return NULL;
}

QrmiQuantumResource *qrmi_resource_new(const char *resource_id, QrmiResourceType resource_type) {
    UNUSED_PARAM(resource_type);
    return (QrmiQuantumResource *)_resource(resource_id);
}

QrmiReturnCode qrmi_resource_free(QrmiQuantumResource *ptr) {
    UNUSED_PARAM(ptr);
    return QRMI_RETURN_CODE_SUCCESS;
}

QrmiReturnCode qrmi_resource_is_accessible(QrmiQuantumResource *qrmi, bool *outp) {
    *outp = ((fake_resource_t *)qrmi)->accessible;
    return QRMI_RETURN_CODE_SUCCESS;
}

QrmiReturnCode qrmi_resource_acquire(QrmiQuantumResource *qrmi, char **acquisition_token) {
    fake_resource_t *fake = (fake_resource_t *)qrmi;
    if (!fake->acquire_ok) {
        return QRMI_RETURN_CODE_ERROR;
    }
    char token[64];
    snprintf(token, sizeof(token), "token-%s-%d", fake->name, ++fake->acquired);
    *acquisition_token = _qrmi_strdup(token);
    return QRMI_RETURN_CODE_SUCCESS;
}

QrmiReturnCode qrmi_resource_release(QrmiQuantumResource *qrmi, const char *acquisition_token) {
    fake_resource_t *fake = (fake_resource_t *)qrmi;
    snprintf(fake->released_token, sizeof(fake->released_token), "%s", acquisition_token);
    return QRMI_RETURN_CODE_SUCCESS;
}

/* --------------------------------------------------------------------- tests */

static int g_failures = 0;

#define CHECK(cond)                                                                \
    do {                                                                           \
        if (!(cond)) {                                                             \
            fprintf(stderr, "%s:%d: check failed: %s\n", __FILE__, __LINE__, #cond); \
            g_failures++;                                                          \
        }                                                                          \
    } while (0)

#define CHECK_STR(actual, expected)                                                    \
    do {                                                                               \
        const char *_a = (actual);                                                     \
        if (_a == NULL || strcmp(_a, (expected)) != 0) {                               \
            fprintf(stderr, "%s:%d: %s is \"%s\", expected \"%s\"\n", __FILE__, __LINE__, \
                    #actual, _a ? _a : "(null)", (expected));                          \
            g_failures++;                                                              \
        }                                                                              \
    } while (0)

static char *g_default_argv[] = {(char *)"/etc/slurm/qrmi_config.json"};

/*
 * Run the plugin hooks slurmstepd calls before the batch script starts, for a job
 * submitted with `--qpu=<qpu>` (or without --qpu if NULL). Returns the result of
 * slurm_spank_task_init(), which fails the job on SLURM_ERROR.
 */
static int start_job(const char *qpu, int argc, char **argv) {
    CHECK(slurm_spank_init(NULL, 0, NULL) == SLURM_SUCCESS);
    if (qpu != NULL) {
        CHECK(_qpu_names_opt_cb(0, qpu, 1) == SLURM_SUCCESS);
    }
    CHECK(slurm_spank_init_post_opt(NULL, argc, argv) == SLURM_SUCCESS);
    return slurm_spank_task_init(NULL, 0, NULL);
}

static int start_default_job(const char *qpu) { return start_job(qpu, 1, g_default_argv); }

static void end_job(void) { CHECK(slurm_spank_exit(NULL, 0, NULL) == SLURM_SUCCESS); }

static void test_no_qpu_option(void) {
    CHECK(start_default_job(NULL) == SLURM_SUCCESS);
    CHECK(g_resources[0].acquired == 0);
    CHECK(job_env_get("QRMI_JOB_QPU_RESOURCES") == NULL);
    end_job();
}

static void test_task_step_is_skipped(void) {
    g_job.stepid = 0; /* srun step inside the allocation, not the batch script */
    start_default_job("local_qpu");
    CHECK(g_resources[0].acquired == 0);
    end_job();
    CHECK_STR(g_resources[0].released_token, "");
}

static void test_missing_config_argument(void) {
    CHECK(start_job("local_qpu", 0, NULL) == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "QRMI config file not specified") != NULL);
    end_job();
}

static void test_config_load_failure(void) {
    g_config_load_fails = true;
    CHECK(start_default_job("local_qpu") == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "Failed to load QRMI config file(/etc/slurm/qrmi_config.json)") !=
          NULL);
    CHECK(g_resources[0].acquired == 0);
    end_job();
}

static void test_single_resource(void) {
    CHECK(start_default_job("local_qpu") == SLURM_SUCCESS);
    CHECK(g_resources[0].acquired == 1);
    CHECK_STR(getenv("local_qpu_QRMI_JOB_ACQUISITION_TOKEN"), "token-local_qpu-1");
    CHECK_STR(job_env_get("local_qpu_QRMI_JOB_ACQUISITION_TOKEN"), "token-local_qpu-1");
    CHECK_STR(job_env_get("local_qpu_QRMI_URL"), "http://config");
    CHECK_STR(job_env_get("QRMI_JOB_QPU_RESOURCES"), "local_qpu");
    CHECK_STR(job_env_get("QRMI_JOB_QPU_TYPES"), "pasqal-local");
    CHECK_STR(job_env_get("SLURM_JOB_QPU_RESOURCES"), "local_qpu");
    CHECK_STR(job_env_get("SLURM_JOB_QPU_TYPES"), "pasqal-local");
    CHECK_STR(job_env_get("QRMI_JOB_ID"), "42");
    CHECK_STR(job_env_get("QRMI_JOB_UID"), "1000");
    CHECK_STR(job_env_get("local_qpu_QRMI_JOB_TIMEOUT_SECONDS"), "600");
    CHECK(job_env_get("cloud_qpu_QRMI_JOB_ACQUISITION_TOKEN") == NULL);
    CHECK_STR(g_job.errors, "");
    end_job();
    CHECK_STR(g_resources[0].released_token, "token-local_qpu-1");
}

static void test_resource_list(void) {
    CHECK(start_default_job("local_qpu,cloud_qpu") == SLURM_SUCCESS);
    CHECK_STR(job_env_get("local_qpu_QRMI_JOB_ACQUISITION_TOKEN"), "token-local_qpu-1");
    CHECK_STR(job_env_get("cloud_qpu_QRMI_JOB_ACQUISITION_TOKEN"), "token-cloud_qpu-1");
    CHECK_STR(job_env_get("QRMI_JOB_QPU_RESOURCES"), "local_qpu,cloud_qpu");
    CHECK_STR(job_env_get("QRMI_JOB_QPU_TYPES"), "pasqal-local,pasqal-cloud");
    CHECK_STR(job_env_get("cloud_qpu_QRMI_JOB_TIMEOUT_SECONDS"), "600");
    end_job();
    CHECK_STR(g_resources[0].released_token, "token-local_qpu-1");
    CHECK_STR(g_resources[1].released_token, "token-cloud_qpu-1");
}

static void test_unknown_resource(void) {
    CHECK(start_default_job("missing_qpu") == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "resource missing_qpu not found") != NULL);
    end_job();
}

static void test_unknown_resource_in_list(void) {
    CHECK(start_default_job("local_qpu,missing_qpu") == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "resource missing_qpu not found") != NULL);
    end_job();
    CHECK_STR(g_resources[0].released_token, "token-local_qpu-1");
}

static void test_inaccessible_resource(void) {
    g_resources[0].accessible = false;
    CHECK(start_default_job("local_qpu") == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "local_qpu is not accessible") != NULL);
    CHECK(g_resources[0].acquired == 0);
    end_job();
}

static void test_acquire_failure(void) {
    g_resources[0].acquire_ok = false;
    CHECK(start_default_job("local_qpu") == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "resource acquisition failed: local_qpu") != NULL);
    CHECK(job_env_get("local_qpu_QRMI_JOB_ACQUISITION_TOKEN") == NULL);
    end_job();
    CHECK_STR(g_resources[0].released_token, "");
}

static void test_plugin_env_arguments(void) {
    char *argv[] = {g_default_argv[0], (char *)"--env:QRMI_TEST_VAR=a=b", (char *)"ignored"};
    CHECK(start_job("local_qpu", 3, argv) == SLURM_SUCCESS);
    CHECK_STR(getenv("QRMI_TEST_VAR"), "a=b");
    CHECK_STR(job_env_get("QRMI_TEST_VAR"), "a=b");
    end_job();
}

static void test_plugin_env_argument_without_equals(void) {
    char *argv[] = {g_default_argv[0], (char *)"--env:QRMI_TEST_VAR"};
    CHECK(start_job("local_qpu", 2, argv) == SLURM_ERROR);
    CHECK(strstr(g_job.errors, "'=' delimiter not found in --env:QRMI_TEST_VAR") != NULL);
    CHECK(g_resources[0].acquired == 0);
    end_job();
}

/* {name}_QRMI_* variables of the job reach the acquire call and take precedence over the config. */
static void test_job_env_is_copied(void) {
    job_env_add("local_qpu_QRMI_URL=http://user");
    job_env_add("cloud_qpu_QRMI_URL=http://other");
    job_env_add("QRMI_UNRELATED=1");
    CHECK(start_default_job("local_qpu") == SLURM_SUCCESS);
    CHECK_STR(getenv("local_qpu_QRMI_URL"), "http://user");
    CHECK_STR(job_env_get("local_qpu_QRMI_URL"), "http://user");
    CHECK(getenv("cloud_qpu_QRMI_URL") == NULL);
    CHECK(getenv("QRMI_UNRELATED") == NULL);
    end_job();
}

static void test_srun_debug_sets_rust_log(void) {
    job_env_add("SRUN_DEBUG=4"); /* srun --verbose */
    CHECK(start_default_job("local_qpu") == SLURM_SUCCESS);
    CHECK_STR(getenv("RUST_LOG"), "debug");
    CHECK_STR(job_env_get("RUST_LOG"), "debug");
    end_job();
}

static void test_job_env_entry_without_equals(void) {
    job_env_add("local_qpu_QRMI_BROKEN");
    job_env_add("local_qpu_QRMI_URL=http://user");
    CHECK(start_default_job("local_qpu") == SLURM_SUCCESS);
    CHECK_STR(getenv("local_qpu_QRMI_URL"), "http://user");
    end_job();
}

static void test_token_freed_with_matching_deallocator(void) {
    CHECK(start_default_job("local_qpu") == SLURM_SUCCESS);
    end_job();
    CHECK(g_foreign_string_frees == 0);
}

/* A token left in the job environment (e.g. sbatch from inside a QPU job) must not win. */
static void test_fresh_token_replaces_job_env_token(void) {
    job_env_add("local_qpu_QRMI_JOB_ACQUISITION_TOKEN=token-of-parent-job");
    CHECK(start_default_job("local_qpu") == SLURM_SUCCESS);
    CHECK_STR(getenv("local_qpu_QRMI_JOB_ACQUISITION_TOKEN"), "token-local_qpu-1");
    CHECK_STR(job_env_get("local_qpu_QRMI_JOB_ACQUISITION_TOKEN"), "token-local_qpu-1");
    end_job();
}

static const struct {
    const char *name;
    void (*run)(void);
} g_tests[] = {
    {"no_qpu_option", test_no_qpu_option},
    {"task_step_is_skipped", test_task_step_is_skipped},
    {"missing_config_argument", test_missing_config_argument},
    {"config_load_failure", test_config_load_failure},
    {"single_resource", test_single_resource},
    {"resource_list", test_resource_list},
    {"unknown_resource", test_unknown_resource},
    {"unknown_resource_in_list", test_unknown_resource_in_list},
    {"inaccessible_resource", test_inaccessible_resource},
    {"acquire_failure", test_acquire_failure},
    {"plugin_env_arguments", test_plugin_env_arguments},
    {"plugin_env_argument_without_equals", test_plugin_env_argument_without_equals},
    {"job_env_is_copied", test_job_env_is_copied},
    {"srun_debug_sets_rust_log", test_srun_debug_sets_rust_log},
    {"job_env_entry_without_equals", test_job_env_entry_without_equals},
    {"token_freed_with_matching_deallocator", test_token_freed_with_matching_deallocator},
    {"fresh_token_replaces_job_env_token", test_fresh_token_replaces_job_env_token},
};

static void _on_alarm(int sig) {
    UNUSED_PARAM(sig);
    static const char msg[] = "timed out, a plugin hook did not return\n";
    UNUSED_PARAM(write(STDERR_FILENO, msg, sizeof(msg) - 1));
    _exit(1);
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s <test case>\n", argv[0]);
        return 2;
    }
    unsetenv("RUST_LOG");
    signal(SIGALRM, _on_alarm);
    alarm(5); /* a hanging hook fails the test instead of blocking CI */
    for (size_t i = 0; i < sizeof(g_tests) / sizeof(g_tests[0]); i++) {
        if (strcmp(g_tests[i].name, argv[1]) == 0) {
            g_tests[i].run();
            if (g_failures > 0) {
                fprintf(stderr, "slurm_error log:\n%s", g_job.errors);
            }
            return g_failures == 0 ? 0 : 1;
        }
    }
    fprintf(stderr, "unknown test case: %s\n", argv[1]);
    return 2;
}
