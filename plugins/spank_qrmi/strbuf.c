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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INITIAL_SIZE 4096

#include "strbuf.h"

int strbuf_init(string_buffer_t *sb) {
    sb->buffer = (char *)malloc(INITIAL_SIZE);
    if (!sb->buffer) {
        perror("malloc");
        return -1;
    }
    sb->capacity = INITIAL_SIZE;
    sb->length = 0;
    sb->buffer[0] = '\0';
    return 0;
}

static void _ensure_capacity(string_buffer_t *sb, size_t additional_len) {
    size_t required = sb->length + additional_len + 1; // +1 for '\0'
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

void strbuf_append_str(string_buffer_t *sb, const char *str) {
    size_t str_len = strlen(str);
    size_t sep_len = (sb->length > 0) ? 1 : 0; // ','
    _ensure_capacity(sb, sep_len + str_len);
    if (sep_len) {
        sb->buffer[sb->length++] = ',';
    }
    memcpy(sb->buffer + sb->length, str, str_len);
    sb->length += str_len;
    sb->buffer[sb->length] = '\0';
}

void strbuf_free(string_buffer_t *sb) {
    free(sb->buffer);
    sb->buffer = NULL;
    sb->capacity = 0;
    sb->length = 0;
}
