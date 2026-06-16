<?php
// With no active output buffer, the level is zero.
echo ob_get_level(), "\n";
// Start the first buffer: level becomes one.
ob_start();
echo ob_get_level(), "\n";
// Start a nested buffer: level becomes two.
ob_start();
echo ob_get_level(), "\n";
// Remove the inner buffer: level returns to one.
ob_end_flush();
echo ob_get_level(), "\n";
// Remove the outer buffer: level returns to zero.
ob_end_flush();
echo ob_get_level(), "\n";
