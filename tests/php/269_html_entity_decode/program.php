<?php
echo 'decoded:[' . html_entity_decode('A &amp; B &lt;tag&gt; &quot;q&quot;') . "]\n";
echo 'exists:[' . function_exists('html_entity_decode') . "]\n";
