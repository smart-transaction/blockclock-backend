. .version
BUILD=$(echo ${BUILD_VERSION} | cut -f 3 -d ".")
VER=$(echo ${BUILD_VERSION} | cut -f 1,2 -d ".")

echo ${BUILD}
echo ${VER}

BUILD=$((${BUILD} + 1))
echo "${VER}.${BUILD}"
