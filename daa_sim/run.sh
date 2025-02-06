#!/bin/sh
#
# (C) Copyright 2024 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.
#
pidfile=".pid"
if [ -r "$pidfile" ]; then
    echo "$pidfile exists: the service probably has already be launched."
    exit 0
fi
python -m direct_access_client.daa_sim.app $@ &
sub_pid=$!
echo $sub_pid > $pidfile
