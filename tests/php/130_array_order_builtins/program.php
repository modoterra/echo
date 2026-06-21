<?php
// array_reverse() returns elements in reverse order.
// Source: https://www.php.net/manual/en/function.array-reverse.php
// array_flip() exchanges int/string values with their original keys.
// Source: https://www.php.net/manual/en/function.array-flip.php
$row = ["sku" => "A-42", 7 => "seven", "qty" => "2", 10 => "ten"];
$reversed = array_reverse($row);
$preserved = array_reverse($row, true);
$map = ["first" => "id", "second" => "qty", "third" => "id", "num" => "2", "int" => 5];
$flipped = array_flip($map);

echo "reverse-keys:[" . implode(",", array_keys($reversed)) . "]\n";
echo "reverse-values:[" . implode(",", array_values($reversed)) . "]\n";
echo "preserve-keys:[" . implode(",", array_keys($preserved)) . "]\n";
echo "preserve-first:[" . array_key_first($preserved) . "]\n";
echo "flip-keys:[" . implode(",", array_keys($flipped)) . "]\n";
echo "flip-id:[" . $flipped["id"] . "]\n";
echo "flip-two:[" . $flipped[2] . "]\n";
echo "flip-five:[" . $flipped[5] . "]\n";
echo "exists:[" . function_exists("array_reverse") . function_exists("array_flip") . "]\n";
