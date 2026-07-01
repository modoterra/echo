<?php
class User {
    public function __construct(public string $name) {}
}

$user = new User("old");
$copy = clone($user, ["name" => "new"]);
echo $copy->name . "\n";
