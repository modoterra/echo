<?php
echo "letters:[" . addcslashes("A\nZ", "A..Z") . "]\n";
echo "controls:[" . addcslashes("A\n\t", "\n\t") . "]\n";
echo "exists:[" . function_exists("addcslashes") . "]\n";
