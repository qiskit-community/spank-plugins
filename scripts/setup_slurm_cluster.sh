#!/bin/bash

# Step 1: Clone the repository to a temporary directory
temp_dir=$(mktemp -d)
echo "Cloning repository to: $temp_dir"
git clone https://github.com/giovtorres/slurm-docker-cluster.git "$temp_dir"

# Step 2: Change directory to the cloned repository
cd "$temp_dir" || { echo "Failed to enter directory."; exit 1; }

# Step 3: Build the Docker containers using Docker Compose
echo "Building Docker containers..."
docker compose build || { echo "Docker build failed."; exit 1; }

# Step 4: Print the containers and volumes information
cat <<EOL

Containers:
  * mysql: Stores job and cluster data.
  * slurmdbd: Manages the Slurm database.
  * slurmctld: The Slurm controller responsible for job and resource management.
  * c1, c2: Compute nodes (running slurmd).

Persistent Volumes:
  * etc_munge: Mounted to /etc/munge
  * etc_slurm: Mounted to /etc/slurm
  * slurm_jobdir: Mounted to /data
  * var_lib_mysql: Mounted to /var/lib/mysql
  * var_log_slurm: Mounted to /var/log/slurm

EOL

# Step 5: Check for running containers with conflicting names and prompt the user
containers=("mysql" "slurmdbd" "slurmctld" "c1" "c2")
running_containers=()

for container_name in "${containers[@]}"; do
  if docker ps --filter "name=$container_name" --format "{{.Names}}" | grep -q "$container_name"; then
    running_containers+=("$container_name")
  fi
done

if [ ${#running_containers[@]} -ne 0 ]; then
  echo "The following containers are currently running: ${running_containers[*]}"
  echo "Do you want to stop them and clean the environment to start from scratch? (y/n)"
  read -r response
  if [[ "$response" == "y" || "$response" == "Y" ]]; then
    echo "Stopping running containers and waiting for termination..."
    docker kill $(docker ps --filter "name=$container_name" --format "{{.ID}}") && docker rm -f $(docker ps -a --filter "name=$container_name" --format "{{.ID}}")

    # Ensure containers are fully stopped before continuing
    for container_name in "${containers[@]}"; do
      while docker ps --filter "name=$container_name" --format "{{.Names}}" | grep -q "$container_name"; do
        echo "Waiting for container $container_name to stop..."
        sleep 2
      done
    done

    echo "Removing containers and volumes..."
    docker compose down -v
  else
    echo "Aborting setup."; exit 1;
  fi
fi

# Step 6: Starting the Cluster
echo "Starting the cluster using docker compose up -d..."

docker compose up -d || {
  echo "Container name conflict detected."
  echo "Do you want to force remove conflicting containers and retry? (y/n)"
  read -r response
  if [[ "$response" == "y" || "$response" == "Y" ]]; then
    echo "Stopping and removing conflicting containers..."
    docker kill $(docker ps --filter "name=$container_name" --format "{{.ID}}") && docker rm -f $(docker ps -a --filter "name=$container_name" --format "{{.ID}}")

    # Ensure containers are fully stopped before continuing
    for container_name in "${containers[@]}"; do
      while docker ps --filter "name=$container_name" --format "{{.Names}}" | grep -q "$container_name"; do
        echo "Waiting for container $container_name to stop..."
        sleep 2
      done
    done

    docker compose down -v
    echo "Retrying docker compose up..."
    docker compose up -d || { echo "Failed to start containers again."; exit 1; }
  else
    echo "Aborting setup."; exit 1;
  fi
}

# Step 7: Sleep for 30 seconds and display progress bar
echo "Waiting for readiness..."
for i in {1..30}; do
  sleep 1
  echo -ne "Progress: $((i * 100 / 30))%\r"
done
echo -e "\nStartup complete."

# Step 8: Display status of the running containers
echo "Displaying container statuses:"
docker compose ps

# Step 9: Registering the cluster with SlurmDBD
echo -e "\n📝 Registering the Cluster with SlurmDBD..."
./register_cluster.sh || { echo "Cluster registration failed. Proceeding to next step..."; }

# Step 10: Follow container logs interactively
echo "Following container logs interactively..."
docker compose logs -f

# Step 11: Open a shell in the slurmctld container and run sinfo
echo "Opening a shell in the slurmctld container and running sinfo..."
docker exec -it slurmctld bash -c "sinfo"
