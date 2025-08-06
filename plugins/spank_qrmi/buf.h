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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *buffer;
    size_t capacity;
} buffer;

extern int  qrmi_buf_init(buffer *, size_t);
extern char* qrmi_buf_envvarname_for_res_create(buffer *, char*, char*);
extern void qrmi_buf_free(buffer *);
