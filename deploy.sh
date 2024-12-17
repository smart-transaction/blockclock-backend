# Full stxn solver setup on a clean Linux machine.
#
# Pre-reqs:
# 1. Linux machine: Debian/Ubuntu/...
# 2. setup.sh file from our setup folder locally in a local folder
#    (pulled from Github or otherwise).

set -e

# Choose the environment
PS3="Please choose the environment: "
options=("dev" "prod" "quit")
select OPT in "${options[@]}"
do
  case ${OPT} in
    "dev")
        echo "Using dev environment"
        TIME_WINDOW="2s"
        WS_CHAIN_URL="wss://service.lestnet.org:8888"
        TICK_PERIOD="2s"
        break
        ;;
    "prod")
        echo "Prod environment is not implemented"
        exit
        ;;
    "quit")
        exit
        ;;
    *) echo "invalid option $REPLY";;
  esac
done

SECRET_SUFFIX=$(echo ${OPT} | tr '[a-z]' '[A-Z]')

# Create necessary files.
cat >up.sh << UP
# Turn up solver.

# Secrets
cat >.env << ENV
MYSQL_ROOT_PASSWORD=\$(gcloud secrets versions access 1 --secret="MYSQL_ROOT_PASSWORD_${SECRET_SUFFIX}")
MYSQL_APP_PASSWORD=\$(gcloud secrets versions access 1 --secret="MYSQL_APP_PASSWORD_${SECRET_SUFFIX}")
MYSQL_READER_PASSWORD=\$(gcloud secrets versions access 1 --secret="MYSQL_READER_PASSWORD_${SECRET_SUFFIX}")
SOLVER_PRIVATE_KEY=\$(gcloud secrets versions access 1 --secret="BLOCKCLOCK_WALLET_PRIVATE_KEY_${SECRET_SUFFIX}")

ENV

sudo docker-compose up -d --remove-orphans

rm -f .env

UP

sudo chmod a+x up.sh

cat >down.sh << DOWN
# Turn down solver.
sudo docker-compose down
DOWN
sudo chmod a+x down.sh

# Docker images
DOCKER_LOCATION="us-central1-docker.pkg.dev"
DOCKER_PREFIX="${DOCKER_LOCATION}/solver-438012/solver-docker-repo"
SOLVER_DOCKER_IMAGE="${DOCKER_PREFIX}/blockclock-solver-image:${OPT}"
DB_DOCKER_IMAGE="${DOCKER_PREFIX}/blockclock-db-image:live"

# Create docker-compose.yml file.
cat >docker-compose.yml << COMPOSE
version: '3'

services:
  blockclock_solver:
    container_name: blockclock_solver
    image: ${SOLVER_DOCKER_IMAGE}
    environment:
      - MYSQL_URL=mysql://server:\${MYSQL_APP_PASSWORD}@blockclock_db:3306/timekeeper
      - TIME_WINDOW=${TIME_WINDOW}
      - SOLVER_PRIVATE_KEY=\${SOLVER_PRIVATE_KEY}
      - WS_CHAIN_URL=${WS_CHAIN_URL}
      - TICK_PERIOD=${TICK_PERIOD}
    ports:
      - 8000:8000

  blockclock_db:
    container_name: blockclock_db
    image: ${DB_DOCKER_IMAGE}
    environment:
      - MYSQL_ROOT_PASSWORD=\${MYSQL_ROOT_PASSWORD}
      - MYSQL_APP_PASSWORD=\${MYSQL_APP_PASSWORD}
      - MYSQL_READER_PASSWORD=\${MYSQL_READER_PASSWORD}
    volumes:
      - mysql:/var/lib/mysql
    ports:
      - 3306:3306

volumes:
  mysql:

COMPOSE

set -e

# Pull images:
docker pull ${SOLVER_DOCKER_IMAGE}
docker pull ${DB_DOCKER_IMAGE}

# Start our docker images.
./up.sh