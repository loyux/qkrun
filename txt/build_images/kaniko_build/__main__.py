import pulumi
import pulumi_kubernetes as k8s
# Create resources from standard Kubernetes guestbook YAML example.
# os.system("kubectl create configmap docker-config --from-file=venv/docker-config.json")
sources = "git"


if sources == "git":
    kaniko_build = k8s.yaml.ConfigFile('build_pod', 'kaniko.yaml')
if sources == "local":
    print(1)
# Export the private cluster IP address of the frontend.
# frontend = guestbook.get_resource('v1/Service', 'frontend')