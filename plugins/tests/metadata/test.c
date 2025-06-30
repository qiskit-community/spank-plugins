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
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>

int main(int argc, char** argv)
{
    void *handle;
    char *type, *name;
    uint32_t *version;

    if (argc != 2) {
        printf("(E) Missing argument. Specify path to plugin library file.\n");
        exit(EXIT_FAILURE);
    }

    handle = dlopen(argv[1], RTLD_LAZY);
    if (handle == NULL) {
        printf("(E) %s\n", dlerror());
        exit(EXIT_FAILURE);
    }

    if (!(name = dlsym(handle, "plugin_name"))) {
        printf("(E) `plugin_name` symbol not found in %s: %s.\n", argv[1], dlerror());
        goto error;
    }
    if (!(type = dlsym(handle, "plugin_type"))) {
        printf("(E) `plugin_type` symbol not found in %s: %s.\n", argv[1], dlerror());
        goto error;
    }
    if (!(version = dlsym(handle, "plugin_version"))) {
        printf("(E) `plugin_version` symbol not found in %s: %s.\n", argv[1], dlerror());
        goto error;
    }

    printf("Valid Slurm plugin library. name=%s, type=%s, version=0x%x\n", name, type, version);
    dlclose(handle);

    return 0;

error:
    dlclose(handle);
    exit(EXIT_FAILURE);
}
