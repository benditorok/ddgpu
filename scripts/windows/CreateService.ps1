# Define variables
$serviceName = "ddgpu"
$executablePath = "C:\services\ddgpu\ddgpu.exe"
$arguments = "--name NVIDIA GeForce RTX 3050 Laptop GPU --hide false --as-service true"

# Define service description
$description = "Disable dedicated GPU service. Runs as admin."

# Install the service using New-Service
New-Service -Name $serviceName -BinaryPathName "$executablePath $arguments" -Description $description -DisplayName "ddgpu" -StartupType Automatic

# Set the service to run as LocalSystem
sc.exe config $serviceName obj= LocalSystem

# Start the service
Start-Service -Name $serviceName
