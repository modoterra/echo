<?php
echo "mail:" . mail("nobody@example.test", "Subject", "Body") . "\n";
echo "exists:" . function_exists("mail") . "\n";
