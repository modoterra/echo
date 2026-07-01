<?php
echo "http_tcp:[" . getservbyname("http", "tcp") . "]\n";
echo "domain_udp:[" . getservbyname("domain", "udp") . "]\n";
echo "missing_type:[" . gettype(getservbyname("definitely-not-a-service", "tcp")) . "]\n";
echo "exists:[" . function_exists("getservbyname") . "]\n";
