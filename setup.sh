#!/bin/bash

# Обновление и апгрейд системы
echo "Обновление системы..."
sudo apt-get update -y
sudo apt-get upgrade -y

# Установка screen
echo "Установка screen..."
sudo apt-get install screen -y

# Установка зависимостей для Rust
echo "Установка зависимостей для Rust..."
sudo apt-get install curl build-essential -y

# Установка Rust и Cargo
echo "Установка Rust и Cargo..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Установка драйверов NVIDIA
echo "Установка драйверов NVIDIA..."
sudo apt-get install -y software-properties-common
sudo add-apt-repository ppa:graphics-drivers/ppa -y
sudo apt-get update -y
sudo apt-get install -y nvidia-driver-525  # Укажите актуальную версию драйвера
sudo apt-get install -y nvidia-settings

# Установка CUDA Toolkit
echo "Установка CUDA Toolkit..."
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-ubuntu2204.pin
sudo mv cuda-ubuntu2204.pin /etc/apt/preferences.d/cuda-repository-pin-600
sudo apt-key adv --fetch-keys https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/7fa2af80.pub
sudo add-apt-repository "deb https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/ /"
sudo apt-get update -y
sudo apt-get install -y cuda

# Настройка окружения для CUDA
echo "Настройка окружения для CUDA..."
echo 'export PATH=/usr/local/cuda/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc

# Установка зависимостей для работы с Tron и GPU
echo "Установка дополнительных зависимостей..."
sudo apt-get install -y protobuf-compiler libssl-dev pkg-config

# Установка библиотеки OpenCL (опционально, для работы с GPU)
echo "Установка OpenCL..."
sudo apt-get install -y ocl-icd-opencl-dev

# Установка tron-rs и других зависимостей через Cargo
echo "Установка зависимостей Rust..."
cargo install tron-rs
cargo install rust-cuda

# Установка dotenv для работы с переменными окружения
echo "Установка dotenv..."
cargo install dotenv

# Завершение
echo "Установка завершена!"
echo "Перезагрузите систему для применения изменений: sudo reboot"
