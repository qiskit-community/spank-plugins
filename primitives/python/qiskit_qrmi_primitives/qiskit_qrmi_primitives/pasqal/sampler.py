# This code is part of Qiskit.
#
# (C) Copyright 2025 Pasqal, IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Sampler V2 base class for Pasqal Cloud QRMI"""
from dataclasses import dataclass, field
from typing import Iterable

from qiskit_pasqal_provider.providers import Sampler
from qiskit_pasqal_provider.providers.abstract_base import PasqalJob

from qrmi import QuantumResource


@dataclass
class Options:
    """Options for :class:`~.QPPSamplerV2`"""

    default_shots: int = 1024
    """The default shots to use if none are specified in :meth:`~.run`.
    Default: 1024.
    """

    run_options: dict = field(default_factory=dict)
    """run_options: Options passed to run.
    """


class QPPSamplerV2(Sampler):
    """Sampler V2 base class for Pasqal QPUs
    Note: future work to make it work with BaseSamplerV2"""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        self._qrmi = qrmi
        self._options = Options(**options) if options else Options()

    def run(
        self, pubs: Iterable[QuantumCircuittes], shots: int | None = None
    ) -> PasqalJob:
        raise NotImplementedError
