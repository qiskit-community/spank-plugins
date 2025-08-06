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

#include "buf.h"

int qrmi_buf_init(buffer *sb, size_t size)
{
    sb->buffer = (char *)malloc(size);
    if (!sb->buffer) {
        perror("malloc");
        return -1;
    }
    sb->capacity = size;
    sb->buffer[0] = '\0';
    return 0;
}

static void _ensure_capacity(buffer *sb, size_t required)
{
    if (required > sb->capacity) {
        size_t new_capacity = sb->capacity;
        while (new_capacity < required) {
            new_capacity *= 2;
        }
        char *new_buffer = (char *)realloc(sb->buffer, new_capacity);
        if (!new_buffer) {
            perror("realloc");
            free(sb->buffer);
            exit(1);
        }
        sb->buffer = new_buffer;
        sb->capacity = new_capacity;
    }
}

char* qrmi_buf_envvarname_for_res_create(buffer *sb, char* resource_id, char* envvar_name)
{
    sb->buffer[0] = '\0';
    size_t keylen = strlen(resource_id) + strlen(envvar_name) + 1;
    _ensure_capacity(sb, keylen + 1);
    snprintf(sb->buffer, sb->capacity, "%s_%s", resource_id, envvar_name);
    return sb->buffer;
}

void qrmi_buf_free(buffer *sb)
{
    free(sb->buffer);
    sb->buffer = NULL;
    sb->capacity = 0;
}
