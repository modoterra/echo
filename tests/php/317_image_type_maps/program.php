<?php
echo image_type_to_extension(1) . "\n";
echo image_type_to_extension(2, false) . "\n";
echo image_type_to_extension(3) . "\n";
echo image_type_to_mime_type(1) . "\n";
echo image_type_to_mime_type(2) . "\n";
echo image_type_to_mime_type(3) . "\n";
echo image_type_to_mime_type(18) . "\n";
echo "bad-ext:" . image_type_to_extension(999) . "\n";
echo "exists:" . function_exists("image_type_to_extension") . function_exists("image_type_to_mime_type") . "\n";
