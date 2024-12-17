PORT=8000
MYSQL_URL=mysql://server:secret2@localhost:3306/timekeeper
CHAIN_ID=21363
TIME_WINDOW=1s
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
  --mysql-url=${MYSQL_URL} \
  --chain-id=${CHAIN_ID} \
  --time-window=${TIME_WINDOW} \
  --solver-private-key=${SOLVER_PRIVATE_KEY} \
  --ws-chain-url=${WS_CHAIN_URL} \
  --tick-period=${TICK_PERIOD}
