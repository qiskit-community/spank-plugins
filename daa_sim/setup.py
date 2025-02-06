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

"""Setup ibm-qcsc-sdk"""
import os
import setuptools

# Handle version.
VERSION_PATH = os.path.join(os.path.dirname(__file__), "VERSION.txt")
with open(VERSION_PATH, "r") as version_file:
    VERSION = version_file.read().strip()

# Read long description from README.
README_PATH = os.path.join(os.path.abspath(os.path.dirname(__file__)), "README.md")
with open(README_PATH) as readme_file:
    README = readme_file.read()

# Read dependencies from requirements.txt
REQUIREMENTS_PATH = os.path.join(
    os.path.abspath(os.path.dirname(__file__)), "requirements.txt"
)
with open(REQUIREMENTS_PATH) as requirements_file:
    REQUIREMENTS = requirements_file.read()

setuptools.setup(
    name="direct-access",
    version=VERSION,
    description="Direct Access API Simulator",
    long_description=README,
    long_description_content_type="text/markdown",
    url="https://quantum.ibm.com/",
    author="IBM Quantum",
    author_email="qiskit@us.ibm.com",
    license="Apache 2.0",
    classifiers=[
        "Environment :: Console",
        "Intended Audience :: Developers",
        "Intended Audience :: Science/Research",
        "Operating System :: Microsoft :: Windows",
        "Operating System :: MacOS",
        "Operating System :: POSIX :: Linux",
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Scientific/Engineering",
    ],
    keywords="quantum",
    packages=setuptools.find_packages(exclude=["test*", "tools*"]),
    install_requires=REQUIREMENTS,
    package_data={"": ["logging.yaml"]},
    include_package_data=True,
    python_requires=">=3.11",
    zip_safe=False,
    entry_points={
        "console_scripts": [
            "daa_sim = direct_access_client.daa_sim.app:main",
        ]
    },
)
