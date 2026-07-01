<?php
echo "method-missing:[" . method_exists("DefinitelyMissingEchoClass", "run") . "]\n";
echo "property-missing:[" . property_exists("DefinitelyMissingEchoClass", "value") . "]\n";
echo "is-a-missing:[" . is_a("DefinitelyMissingEchoClass", "Base", true) . "]\n";
echo "subclass-missing:[" . is_subclass_of("DefinitelyMissingEchoClass", "Base", true) . "]\n";
echo "exists:[" . function_exists("method_exists") . function_exists("property_exists") . function_exists("is_a") . function_exists("is_subclass_of") . "]\n";
