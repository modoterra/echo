<?php
echo "enabled0:[" . gc_enabled() . "]\n";
gc_disable();
echo "enabled1:[" . gc_enabled() . "]\n";
gc_enable();
echo "enabled2:[" . gc_enabled() . "]\n";
echo "collect:[" . gc_collect_cycles() . "]\n";
echo "mem-int:[" . is_int(gc_mem_caches()) . "]\n";
$status = gc_status();
echo "status:[" . is_array($status) . array_key_exists("runs", $status) . array_key_exists("collected", $status) . array_key_exists("roots", $status) . array_key_exists("application_time", $status) . "]\n";
echo "exists:[" . function_exists("gc_enabled") . function_exists("gc_disable") . function_exists("gc_enable") . function_exists("gc_collect_cycles") . function_exists("gc_mem_caches") . function_exists("gc_status") . "]\n";
