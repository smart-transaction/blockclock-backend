PORT=8000
MYSQL_USER=server
MYSQL_PASSWORD=secret2
MYSQL_HOST=localhost
MYSQL_PORT=3306
MYSQL_DATABASE=timekeeper
TIME_WINDOW=3s
PRIMARY_CHAIN_ID=21363
PRIMARY_HTTP_CHAIN_URL=https://service.lestnet.org
PRIMARY_BLOCK_TIME_ADDRESS=0xdD1B4D9337D0a8Ef2F133a39cC93EF85261b4A80
SECONDARY_CHAIN_ID=84532
SECONDARY_HTTP_CHAIN_URL=https://sepolia.base.org
SECONDARY_BLOCK_TIME_ADDRESS=0xdD1B4D9337D0a8Ef2F133a39cC93EF85261b4A80
TICK_PERIOD=1s

PROJECT_NAME="solver-438012"
CURRENT_PROJECT=$(gcloud config get project)
if [ "${PROJECT_NAME}" != "${CURRENT_PROJECT}" ]; then
  gcloud auth login
  gcloud config set project ${PROJECT_NAME}
fi

SOLVER_PRIVATE_KEY=$(gcloud secrets versions access 1 --secret="BLOCKCLOCK_WALLET_PRIVATE_KEY_PROD")

cargo run \
  -- \
  --port=${PORT} \
  --time-window=${TIME_WINDOW} \
  --solver-private-key=${SOLVER_PRIVATE_KEY} \
  --tick-period=${TICK_PERIOD} \
  --mysql-user=${MYSQL_USER} \
  --mysql-password=${MYSQL_PASSWORD} \
  --mysql-host=${MYSQL_HOST} \
  --mysql-port=${MYSQL_PORT} \
  --mysql-database=${MYSQL_DATABASE} \
  --primary-chain-id=${PRIMARY_CHAIN_ID} \
  --primary-http-chain-url=${PRIMARY_HTTP_CHAIN_URL} \
  --primary-block-time-address=${PRIMARY_BLOCK_TIME_ADDRESS} \
  --secondary-chain-id=${SECONDARY_CHAIN_ID} \
  --secondary-http-chain-url=${SECONDARY_HTTP_CHAIN_URL} \
  --secondary-block-time-address=${SECONDARY_BLOCK_TIME_ADDRESS} \
  --dry-run=true