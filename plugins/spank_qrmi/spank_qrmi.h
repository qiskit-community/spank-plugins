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

#ifndef _SPANK_QRMI_H
#define _SPANK_QRMI_H

#include "buf.h"
#include "strbuf.h"

#define OVERWRITE 1
#define KEEP_IF_EXISTS 0

/*
 * A record of acquired QPU resource.
 */
typedef struct _qpu_resource {
    char *name;              /* resource identifier */
    QrmiResourceType type;   /* resource type */
    char *type_as_str;       /* resource type as string */
    char *acquisition_token; /* acquisition token returned by QRMI */
} qpu_resource_t;

#endif /* !_SPANK_QRMI_H */
