<?php
echo ob_get_level(), "\n";
ob_start();
echo ob_get_level(), "\n";
ob_start();
echo ob_get_level(), "\n";
ob_end_flush();
echo ob_get_level(), "\n";
ob_end_flush();
echo ob_get_level(), "\n";
