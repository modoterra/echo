<?php
echo "missing:[" . class_exists("DefinitelyMissingEchoClass") . interface_exists("DefinitelyMissingEchoInterface") . trait_exists("DefinitelyMissingEchoTrait") . enum_exists("DefinitelyMissingEchoEnum") . "]\n";
echo "missing-noautoload:[" . class_exists("DefinitelyMissingEchoClass", false) . interface_exists("DefinitelyMissingEchoInterface", false) . trait_exists("DefinitelyMissingEchoTrait", false) . enum_exists("DefinitelyMissingEchoEnum", false) . "]\n";
echo "exists:[" . function_exists("class_exists") . function_exists("interface_exists") . function_exists("trait_exists") . function_exists("enum_exists") . "]\n";
