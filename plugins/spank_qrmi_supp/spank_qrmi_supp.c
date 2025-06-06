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
 * along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
 */
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "slurm/slurm.h"
#include "slurm/spank.h"

/*
 * Supplemental Spank plugin to set environment variables not covered by spank_qrmi plugin
 */
SPANK_PLUGIN(spank_qrmi_supp, 1)

static const size_t DEFAULT_KEYBUF_LEN = 256;
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

  struct spank_option opt = {"qpu", NULL, NULL, 1, 0, NULL};
  char *optargp = NULL;
  rc = spank_option_getopt(spank_ctxt, &opt, &optargp);
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
    if (slurm_load_job(&job_info_msg, job_id, SHOW_DETAIL) == SLURM_SUCCESS) {
      /*
       * Slurm's time limit is represented in minutes
       */
      uint32_t time_limit_mins = job_info_msg->job_array[0].time_limit;
      /*
       * Convert minutes to seconds, uint32_t to char*
       */
      memset(limit_as_str, '\0', sizeof(limit_as_str));
      snprintf(limit_as_str, sizeof(limit_as_str), "%u", time_limit_mins * 60);
    }
  }

  if (strlen(limit_as_str) == 0) {
    /* time limit should be there, somthing wrong in Slurm */
    return ESPANK_ERROR;
  }

  /*
   * Set environment variables for each QPU. For example,
   *
   * Following varables are set if '--qpu=qpu1,qpu2' is specified as SBATCH
   * option.
   * - qpu1_QRMI_JOB_TIMEOUT_SECONDS=val
   * - qpu2_QRMI_JOB_TIMEOUT_SECONDS=val
   */
  char *bufp = (char *)malloc(optlen + 1);
  char *rest = bufp;
  char *token;
  char *keybufp = (char *)malloc(DEFAULT_KEYBUF_LEN + 1);
  size_t keybuf_len = DEFAULT_KEYBUF_LEN;
  /*
   * Copy option string to bufp because subsequent strtok_r will
   * modify the source buffer.
   */
  bufp[optlen] = '\0';
  strncpy(bufp, optargp, optlen);

  while ((token = strtok_r(rest, ", ", &rest))) {
    size_t keylen = strlen(token) + strlen("QRMI_JOB_TIMEOUT_SECONDS") + 2;
    if (keylen > keybuf_len) {
      keybufp = (char *)realloc(keybufp, keylen + 1);
      keybuf_len = keylen + 1;
    }
    memset(keybufp, '\0', keybuf_len);
    snprintf(keybufp, keylen, "%s_QRMI_JOB_TIMEOUT_SECONDS", token);
    spank_setenv(spank_ctxt, keybufp, limit_as_str, 1);
  }
  free(bufp);
  free(keybufp);

  slurm_debug("%s(%d,%d): <- %s rc=%d", plugin_name, pid, uid, __FUNCTION__, rc);

  return rc;
}
