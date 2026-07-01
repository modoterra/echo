<?php
error_reporting(0);
echo "trigger:[" . trigger_error("quiet trigger", 1024) . "]\n";
echo "user:[" . user_error("quiet user", 1024) . "]\n";
echo "trigger-exists:[" . function_exists("trigger_error") . "]\n";
echo "user-exists:[" . function_exists("user_error") . "]\n";
