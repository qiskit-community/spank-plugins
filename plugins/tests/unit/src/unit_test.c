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
#include <dlfcn.h>
#include <stdlib.h>

typedef void (*generic_func)();

int main(int argc, char** argv)
{
    void *handle;
    char *error;
 
    // List of functions to check
    const char *functions[] = {
        "slurm_spank_exit",
        "slurm_spank_init",
        "slurm_spank_init_post_opt",
        "slurm_spank_task_init"};
    int num_functions = sizeof(functions) / sizeof(functions[0]);

    if (argc != 2) {
        printf("(Error) Missing argument. Specify path to plugin library file.\n");
        exit(EXIT_FAILURE);
    }

    handle = dlopen(argv[1], RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "Could not open %s: %s\n", argv[1], dlerror());
        exit(EXIT_FAILURE);
    }
    //printf("Starting existence tests for %s ...\n", argv[1]);

    for (int i = 0; i < num_functions; i++) {
        dlerror(); // clear any existing error just in case
        generic_func func_ptr = (generic_func)dlsym(handle, functions[i]);
        error = dlerror();

        if (error != NULL) {
            printf("[FAILED] Function '%s' NOT found. Error: %s\n", functions[i], error);
            exit(EXIT_FAILURE);
        } else if (func_ptr == NULL) {
            // In rare cases, a symbol can exist but have a NULL value
            printf("[WARNING] Function '%s' found but is NULL.\n", functions[i]);
            exit(EXIT_FAILURE);
        } else {
            printf("[PASSED] Function '%s' exists at address %p.\n", functions[i], (void*)func_ptr);
        }
    }
    
    dlclose(handle);
    return (EXIT_SUCCESS);
}
