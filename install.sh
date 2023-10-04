#!/bin/sh

set -e

owner="daxartio"
repo="kdbx"
exe_name="kdbx"
githubUrl=""
githubApiUrl=""
version="0.5.0"

get_arch() {
    # darwin/amd64: Darwin axetroydeMacBook-Air.local 20.5.0 Darwin Kernel Version 20.5.0: Sat May  8 05:10:33 PDT 2021; root:xnu-7195.121.3~9/RELEASE_X86_64 x86_64
    # linux/amd64: Linux test-ubuntu1804 5.4.0-42-generic #46~18.04.1-Ubuntu SMP Fri Jul 10 07:21:24 UTC 2020 x86_64 x86_64 x86_64 GNU/Linux
    a=$(uname -m)
    case ${a} in
        "x86_64" | "amd64" )
            echo "x86_64"
        ;;
        "aarch64" | "arm64" | "arm")
            echo "aarch64"
        ;;
        *)
            echo ${NIL}
        ;;
    esac
}

get_os(){
    # darwin: Darwin
    o=$(uname -s | awk '{print tolower($0)}')
    case ${o} in
        "darwin" )
            echo "apple-darwin"
        ;;
        "linux" )
            echo "unknown-linux-musl"
        ;;
        *)
            echo ${NIL}
        ;;
    esac
}

# parse flag
for i in "$@"; do
    case $i in
        -v=*|--version=*)
            version="${i#*=}"
            shift # past argument=value
        ;;
        *)
            # unknown option
        ;;
    esac
done

if [ -z "$githubUrl" ]; then
    githubUrl="https://github.com"
fi
if [ -z "$githubApiUrl" ]; then
    githubApiUrl="https://api.github.com"
fi

download_folder="${HOME}/Downloads"
mkdir -p ${download_folder}
os=$(get_os)
arch=$(get_arch)
folder_name="${exe_name}-${version}-${arch}-${os}"
file_name_gz="${folder_name}.tar.gz"
downloaded_file="${download_folder}/${file_name_gz}"
extracted_folder="${download_folder}/${folder_name}"
executable_folder="/usr/local/bin"

# if version is empty
if [ -z "$version" ]; then
    asset_path=$(
        command curl -L \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            ${githubApiUrl}/repos/${owner}/${repo}/releases |
        command grep -o "/${owner}/${repo}/releases/download/.*/${file_name_gz}" |
        command head -n 1
    )
    if [[ ! "$asset_path" ]]; then
        echo "ERROR: unable to find a release asset called ${file_name_gz}"
        exit 1
    fi
    asset_uri="${githubUrl}${asset_path}"
else
    asset_uri="${githubUrl}/${owner}/${repo}/releases/download/${version}/${file_name_gz}"
fi

echo "[1/3] Download ${asset_uri} to ${download_folder}"
rm -f ${downloaded_file}
curl --fail --location --output "${downloaded_file}" "${asset_uri}"

echo "[2/3] Install ${exe_name} to the ${executable_folder}"
tar -xz -f ${downloaded_file} -C ${download_folder}
mv ${extracted_folder}/${exe_name} ${executable_folder}
rm ${downloaded_file}
rm -r ${extracted_folder}
exe=${executable_folder}/${exe_name}
chmod +x ${exe}

echo "[3/3] Set environment variables"
echo "${exe_name} was installed successfully to ${exe}"
if command -v $exe_name --version >/dev/null; then
    echo "Run '$exe_name --help' to get started"
else
    echo "Manually add the directory to your \$HOME/.bash_profile (or similar)"
    echo "  export PATH=${executable_folder}:\$PATH"
    echo "Run '$exe_name --help' to get started"
fi

exit 0
