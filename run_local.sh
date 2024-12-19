PORT=8000
MYSQL_URL=mysql://server:secret2@localhost:3306/timekeeper
MYSQL_USER=server
MYSQL_PASSWORD=secret2
MYSQL_HOST=localhost
MYSQL_PORT=3306
MYSQL_DATABASE=timekeeper
CHAIN_ID=21363
TIME_WINDOW=3s
WS_CHAIN_URL=wss://service.lestnet.org:8888/
TICK_PERIOD=1s

PROJECT_NAME="solver-438012"
CURRENT_PROJECT=$(gcloud config get project)
if [ "${PROJECT_NAME}" != "${CURRENT_PROJECT}" ]; then
  gcloud auth login
  gcloud config set project ${PROJECT_NAME}
fi

SOLVER_PRIVATE_KEY=$(gcloud secrets versions access 1 --secret="LOCAL_BLOCKCLOCK_WALLET_PRIVATE_KEY_DEV")

cargo run \
  -- \
  --port=${PORT} \
  --chain-id=${CHAIN_ID} \
  --time-window=${TIME_WINDOW} \
  --solver-private-key=${SOLVER_PRIVATE_KEY} \
  --ws-chain-url=${WS_CHAIN_URL} \
  --tick-period=${TICK_PERIOD} \
  --mysql-user=${MYSQL_USER} \
  --mysql-password=${MYSQL_PASSWORD} \
  --mysql-host=${MYSQL_HOST} \
  --mysql-port=${MYSQL_PORT} \
  --mysql-database=${MYSQL_DATABASE}