#!/usr/bin/env python
# -*- coding: utf-8 -*-

# (C) Copyright 2024, 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""counting samples as post processing of sampler job"""

# pylint: disable=invalid-name, duplicate-code
import argparse
import json

def _main():
    """app main"""
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("output")
    args = parser.parse_args()

    with open(args.input, "r", encoding="utf-8") as input_file:
        results = json.load(input_file).get("results")
        counts_all = []
        for result in results:
            counts = {}
            num_qubits = result["data"]["meas"]["num_bits"]
            for sample in result["data"]["meas"]["samples"]:
                # converts hex string to bits string
                sample = format(int(sample, 16), f"0>{num_qubits}b")
                if sample not in counts:
                    counts[sample] = 0
                counts[sample] += 1
            sorted_counts = dict(sorted(counts.items(), key=lambda x: x[1], reverse=True))
            counts_all.append(sorted_counts)
        with open(args.output, "w", encoding="utf-8") as output_file:
            json.dump(counts_all, output_file, indent=2)

if __name__ == "__main__":
    _main()
