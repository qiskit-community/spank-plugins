import random

from dotenv import load_dotenv
from pulser_qrmi_backend.service import QRMIService
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

# Generate transpiler target from backend configuration & properties
target = get_device(qrmi)
