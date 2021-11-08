# clusterctl
CLI to manage a cluster of machines

This project includes a CLI that allows to easily manage a cluster of machines, and is an
evolution of a set of Bash scripts I had that were becoming too complex. It is based on
[Ansible](https://www.ansible.com), which is an automation tool that offers an easy way to
operate on many different machines at the same time and provides a huge set of built-in
commands for many operations.

# Setup
Because of being based in Ansible, all the operations need an inventory file setup, which
describes all the machines and groups of machines in the cluster.

So, to be able to run it, you'll need the following:
1. A group of machines to manage :D
2. Setup those machines to be able to connect to them and run commands, via Ansible, on those machines:
   1. Right now, only Ubuntu OS is supported, so install that on all the cluster machines. (Support for other OS [is planned](https://github.com/rodrmoya/clusterctl/issues/3)).
   2. Enable SSH on all of the cluster machines.
   3. Create a user on all of the cluster machines with the same name everywhere.
   4. Copy your ~/.ssh/id_rsa* files to the ~/.ssh/ directory on the cluster machines. This is so that you can connect via SSH from your desktop machines.
3. Make sure you can SSH from your "control" machine (the machine where you will be running commands against the cluster, i.e. your desktop machine) to all of the cluster machines.
4. Setup the inventory file on your "control" machine. Follow [this guide](https://docs.ansible.com/ansible/latest/user_guide/intro_inventory.html) for that. You can name your machines as you wish, but at least the inventory should have these:
   1. A `master1` host. This will be the main master host, used for things like setting up the control plane in Kubernetes cluster, for instance.
   2. A `cluster` group, containing all the hosts in your cluster.
   3. A `cluster_managers` group, containing only the hosts that acts as masters in the cluster.
   4. A `cluster_workers` group, containing only the hosts that are not masters, just normal "workers"
   ```
   master1 ansible_host=192.168.0.10
   worker1 ansible_host=192.168.0.11
   worker2 ansible_host=192.168.0.12

   [cluster]
   master1
   worker1
   worker2

   [cluster_managers]
   master1

   [cluster_workers]
   worker1
   worker2
   ```

# Running
```
USAGE:
    clusterctl [FLAGS] [OPTIONS] --inventory <INVENTORY> <SUBCOMMAND>

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Level of verbosity
    -V, --version    Print version information

OPTIONS:
    -i, --inventory <INVENTORY>
            Host inventory file (in Ansible supported format)

    -p, --host-pattern <HOST_PATTERN>
            Host pattern. If not specified, all machines in the cluster is assumed

SUBCOMMANDS:
    help       Print this message or the help of the given subcommand(s)
    ping       Ping all machines in the cluster to check they're alive and reachable
    reboot     Reboot all machines in the cluster
    run        Run a command on all machines in the cluster
    service    Commands to operate services on the cluster
    ssh        Open a secure shell connection to a machine on the cluster
    update     Perform OS and apps updates on all the machines in the cluster
    uptime     Show how long machines in the cluster have been running
```
The `--inventory` argument is required, and should be pointing to the Ansible inventory setup in the previous step.