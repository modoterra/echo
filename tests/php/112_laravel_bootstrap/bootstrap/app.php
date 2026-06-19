<?php

namespace Illuminate\Http {

class Request
{
    public static function capture()
    {
        return new self();
    }
}

}

namespace {

class Application
{
    public function handleRequest($request)
    {
    }
}

return new Application();
}
