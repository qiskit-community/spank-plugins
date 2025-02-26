# Direct Access API for C Examples

## Prerequisites
* Python 3.11 or above
* [nlohmann_json](https://github.com/nlohmann/json) for building [cxx samples](./src).

## Setup

> [!NOTE]
> The Direct Access API for C Example relies on the output of the Rust components in this repository. If you have not yet built these components, please build them first as follows.
>
> ```shell-session
> pushd ../../
> cargo build --release
> popd 
> ```

```shell-session
python3.11 -m venv ~/daapi_c_examples
source ~/daapi_c_examples/bin/activate
pip install --upgrade pip
pip install conan
conan profile detect
conan install conanfile.txt --build=missing
pushd build
cJSON_DIR=./Release/generators cmake .. -DCMAKE_BUILD_TYPE=Release
popd
```

## Build examples
```shell-session
cd build
make clean
make
```
