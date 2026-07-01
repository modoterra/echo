<?php
echo "included-count:[" . count(get_included_files()) . "]\n";
echo "required-count:[" . count(get_required_files()) . "]\n";
echo "exists:[" . function_exists("get_included_files") . function_exists("get_required_files") . "]\n";
