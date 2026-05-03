# Broker
could also be called the execution service.

This is where the local client sends its build requests. in much of the code we will just refer to this as "the server".

There must be exactly one broker per Abrasive cluster. It is the process to which all clients and workers connect.