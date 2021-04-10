# Acticks

acticks is a backtesting application. The application is a webserver, and the goal of 
acticks is to mirror the public api of a brokerage as closely as possible, so that the 
user can use acticks as a drop-in replacement for their actual brokerage. The benefit of 
this is that the user should in theory be able to test against acticks and then switch to
the endpoint to their real brokerage for seamless transition from testing to live trading.

Currently, the library aims to mimic the public api of [Alpaca](https://alpaca.markets/), 
but the aim is to make the broker api an implementation detail and allow users to switch to
whichever brokerage api they prefer.

## acticks?
The code uses a popular just library named [`actix`](https://actix.rs/) for implementing
the [actor model](https://en.wikipedia.org/wiki/Actor_model), and aims to be able to test
"tick-level" data, ergo `acticks`.
