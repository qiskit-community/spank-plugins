# -*- coding: utf-8 -*-

# (C) Copyright 2024 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

# This code is part of Qiskit Runtime.
#
# (C) Copyright IBM, 2024.
#
# pylint: disable=too-many-return-statements
"""
Encoder to use with primitives results
"""

import json
from typing import Any
from datetime import date
import numpy as np
from qiskit.primitives.containers import BitArray, DataBin, PubResult, PrimitiveResult
from qiskit_ibm_runtime.execution_span import (
    DoubleSliceSpan,
    ExecutionSpans,
    SliceSpan,
    TwirledSliceSpan,
)


class StandardJSONEncoder(json.JSONEncoder):
    """Pure JSON Encoder used to return primitives results
    While qiskit-ibm-runtime provides a json encoder, it assumes
    the receiving end uses Python and numpy; this encoder
    returns all results in a (less efficient) standard JSON format
    """

    def default(self, o: Any) -> Any:  # pylint: disable=arguments-differ
        if isinstance(o, date):
            return {"date": o.isoformat()}
        if isinstance(o, complex):
            return {"complex": [o.real, o.imag]}
        if isinstance(o, np.ndarray):
            return o.tolist()
        if isinstance(o, np.number):
            return o.item()
        if isinstance(o, set):
            return list(o)
        if isinstance(o, BitArray):
            flat_samples = [hex(int(bits, 2)) for bits in o.get_bitstrings()]
            samples = np.array(flat_samples).reshape(o.array.shape[:-1]).tolist()
            return {"samples": samples, "num_bits": o.num_bits}
        if isinstance(o, DataBin):
            return {field_name: getattr(o, field_name) for field_name in o._FIELDS}
        if isinstance(o, PubResult):
            return {"data": o.data, "metadata": o.metadata}
        if isinstance(o, PrimitiveResult):
            return {"results": o._pub_results, "metadata": o.metadata}
        if isinstance(o, ExecutionSpans):
            return o._spans
        if isinstance(o, SliceSpan):
            return (
                o.start,
                o.stop,
                {
                    idx: (res_shape, (data_slice.start, data_slice.stop))
                    for idx, (res_shape, data_slice) in o._data_slices.items()
                },
            )
        if isinstance(o, DoubleSliceSpan):
            return (
                o.start,
                o.stop,
                {
                    idx: (shape, (sl1.start, sl1.stop), (sl2.start, sl2.stop))
                    for idx, (shape, sl1, sl2) in o._data_slices.items()
                },
            )
        if isinstance(o, TwirledSliceSpan):
            return (
                o.start,
                o.stop,
                {
                    idx: (shape, order, (sl1.start, sl1.stop), (sl2.start, sl2.stop))
                    for idx, (shape, order, sl1, sl2) in o._data_slices.items()
                },
            )

        return super().default(o)
