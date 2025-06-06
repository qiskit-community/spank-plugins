import random

from dotenv import load_dotenv
from pulser import Pulse, QPUBackend, Register, Sequence
from pulser.backend.remote import JobParams
from qrmi_primitives import PulserQRMIConnection, QRMIService
from target import get_device

# Create QRMI
load_dotenv()
service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    raise ValueError("No quantum resource is available.")

# Randomly select QR
qrmi = resources[random.randrange(len(resources))]
print(qrmi.metadata())

qrmi_conn = PulserQRMIConnection(qrmi)

# Generate Pulser device
device = get_device(qrmi)


reg = Register(
    {
        "q0": (-2.5, -2.5),
        "q1": (2.5, -2.5),
        "q2": (-2.5, 2.5),
        "q3": (2.5, 2.5),
    }
)

seq = Sequence(reg, device)
seq.declare_channel("rydberg", "rydberg_global")

pulse1 = Pulse.ConstantPulse(100, 2, 2, 0)

seq.add(pulse1, "rydberg")
seq.measure("ground-rydberg")

backend = QPUBackend(seq, qrmi_conn)
result = backend.run([JobParams(runs=1000, variables=[])], wait=True)
print(result)