# Astroport DCA Module

The DCA contract contains logic to facilitate users performing DCA orders (dollar cost averaging) over a period of time. Users can create DCA orders which will then be fulfilled by another user after enough time has occurred, with them specifying the purchase route from the deposited asset to the target asset. This route can only swap through whitelisted tokens by the contract.

## InstantiateMsg

Initializes the contract with the configuration settings, the [Astroport factory contract](https://github.com/astroport-fi/astroport-core/tree/main/contracts/factory) address and the [Astroport router contract](https://github.com/astroport-fi/astroport-core/tree/main/contracts/router) address. A sample instantiateMsg that we have deploy in our locaterra image `dautt/astroport:v1.2.0` looks as follow:

```json
{
  "owner": "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
  "max_hops": 3,
  "max_spread": "0.5",
  "per_hop_fee": "100000",
  "gas_info": {
    "native_token": {
      "denom": "uluna"
    }
  },
  "whitelisted_tokens": {
    "source": [
      {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      {
        "token": {
          "contract_addr": "terra14haqsatfqxh3jgzn6u7ggnece4vhv0nt8a8ml4rg29mln9hdjfdq9xpv0p"
        }
      },
      {
        "token": {
          "contract_addr": "terra10v0hrjvwwlqwvk2yhwagm9m9h385spxa4s54f4aekhap0lxyekys4jh3s4"
        }
      },
      {
        "native_token": {
          "denom": "uluna"
        }
      }
    ],
    "tip": [
      {
        "token": {
          "contract_addr": "terra1q0e70vhrv063eah90mu97sazhywmeegptx642t5px7yfcrf0rrsq2nesul"
        }
      },
      {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      {
        "native_token": {
          "denom": "uluna"
        }
      }
    ]
  },
  "factory_addr": "terra1qnmggqv3u03axw69cn578hukqhl4f2ze2k403ykcdlhj98978u7stv7fyj",
  "router_addr": "terra15kwtyl2jtf8frwh3zu2jntqvem8u36y8aw6yy9z3ypgkfjx6ct2q73xas8"
}
```

## ExecuteMsg

In this section we provide for each ExecuteMsg a sample which we have deployed in our image `dautt/astroport:v1.2.0`

### `deposit`

After a user has created a dca order, he can deposit more assets (source/tip/gas) into the oder.
Example: In the dca oder with id=2 the user wants to deposit more source asset.

```json
{
  "deposit": {
    "asset": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "200000"
    },
    "dca_order_id": "2",
    "deposit_type": "source"
  }
}
```

### `create_dca_order`

The user can created a new dca order by specifiying the following parameters as schown in this example.

```json
{
  "create_dca_order": {
    "dca_amount": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "1000000"
    },
    "source": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "10000000"
    },
    "gas": {
      "info": {
        "native_token": {
          "denom": "uluna"
        }
      },
      "amount": "1000000"
    },
    "interval": 10,
    "start_at": 1660589561,
    "target_info": {
      "token": {
        "contract_addr": "terra14haqsatfqxh3jgzn6u7ggnece4vhv0nt8a8ml4rg29mln9hdjfdq9xpv0p"
      }
    },
    "tip": {
      "info": {
        "token": {
          "contract_addr": "terra1q0e70vhrv063eah90mu97sazhywmeegptx642t5px7yfcrf0rrsq2nesul"
        }
      },
      "amount": "10000000"
    }
  }
}
```

### `withdraw`

The user can withthdraw one of his assets (source/tip/gas/target) from his dca oder.
Example: the user want to withdraw some of his source asset.

```json
{
  "withdraw": {
    "asset": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "1000000"
    },
    "dca_order_id": "2",
    "withdraw_type": "source"
  }
}
```

### `cancel_dca_oder`

The user can cancel his order.
Example: the user wants to cancel dca oder id=2. All his assets which are tracked in the oder will be refunded.

```json
{
  "cancel_dca_order": {
    "id": "2"
  }
}
```

### `perform_dca_purchase`

Anyone can trigger a oder purchase on behalf of owner od the dca oder.
Example:

```json
{
  "perform_dca_purchase": {
    "dca_order_id": "3",
    "hops": [
      {
        "astro_swap": {
          "offer_asset_info": {
            "token": {
              "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
            }
          },
          "ask_asset_info": {
            "token": {
              "contract_addr": "terra14haqsatfqxh3jgzn6u7ggnece4vhv0nt8a8ml4rg29mln9hdjfdq9xpv0p"
            }
          }
        }
      },
      {
        "astro_swap": {
          "offer_asset_info": {
            "token": {
              "contract_addr": "terra14haqsatfqxh3jgzn6u7ggnece4vhv0nt8a8ml4rg29mln9hdjfdq9xpv0p"
            }
          },
          "ask_asset_info": {
            "native_token": {
              "denom": "uluna"
            }
          }
        }
      }
    ]
  }
}
```

### `modify_dca_oder`

The user can modify his oder as follow. Example:

```json
{
  "modify_dca_order": {
    "id": "1",
    "new_source_asset": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "1000000"
    },
    "new_target_asset_info": {
      "token": {
        "contract_addr": "terra1wastjc07zuuy46mzzl3egz4uzy6fs59752grxqvz8zlsqccpv2wqnfu3yr"
      }
    },
    "new_dca_amount": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "500000"
    },
    "new_tip_asset": {
      "info": {
        "native_token": {
          "denom": "uluna"
        }
      },
      "amount": "64357531"
    },
    "new_interval": 1000,
    "new_start_at": 1000,
    "new_max_hops": 5,
    "new_max_spread": "0.7"
  }
}
```

### `update_config`

Only the owner of the dca contract can updates the contract configuration with the specified optional parameters.

Any parameters that are not specified will be left unchanged.

```json
{
  "update_config": {
    // set max_spread to 0.1
    "max_spread": "0.1",
    // leave max_hops, per_hop_fee, whitelisted_tokens unchanged
    "max_hops": null,
    "per_hop_fee": null,
    "whitelisted_tokens_source": null,
    "whitelisted_tokens_tip": null,
    "max_spread": null,
    "router_addr": null
  }
}
```

## QueryMsg

All query messages are described below.

### `config`

Returns information about the contract configuration (`max_hops`, `max_spread`, etc).

```json
{
  "config": {}
}
```

Example response:

```json
{
  "owner": "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
  "max_hops": 3,
  "max_spread": "0.5",
  "per_hop_fee": "100000",
  "gas_info": {
    "native_token": {
      "denom": "uluna"
    }
  },
  "whitelisted_tokens": {
    "source": [
      {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      {
        "token": {
          "contract_addr": "terra14haqsatfqxh3jgzn6u7ggnece4vhv0nt8a8ml4rg29mln9hdjfdq9xpv0p"
        }
      },
      {
        "token": {
          "contract_addr": "terra10v0hrjvwwlqwvk2yhwagm9m9h385spxa4s54f4aekhap0lxyekys4jh3s4"
        }
      },
      {
        "native_token": {
          "denom": "uluna"
        }
      }
    ],
    "tip": [
      {
        "token": {
          "contract_addr": "terra1q0e70vhrv063eah90mu97sazhywmeegptx642t5px7yfcrf0rrsq2nesul"
        }
      },
      {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      {
        "native_token": {
          "denom": "uluna"
        }
      }
    ]
  },
  "factory_addr": "terra1qnmggqv3u03axw69cn578hukqhl4f2ze2k403ykcdlhj98978u7stv7fyj",
  "router_addr": "terra15kwtyl2jtf8frwh3zu2jntqvem8u36y8aw6yy9z3ypgkfjx6ct2q73xas8"
}
```

### `user_dca_orders`

Returns the list of dca oder ids which belong to a specified user.

```json
{
  "user_dca_orders": { "user": "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v" }
}
```

Example response:

```json
["1", "2"]
```

### `dca_orders`

Returns information about the dca order id.

```json
{
  "dca_orders": {
    "id": "2"
  }
}
```

Example response for two DCA orders:

```json
{
  "id": "2",
  "created_by": "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
  "created_at": 1660596065,
  "start_at": 1660596064,
  "interval": 10,
  "dca_amount": {
    "info": {
      "token": {
        "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
      }
    },
    "amount": "200000"
  },
  "max_hops": 3,
  "max_spread": "0.4",
  "balance": {
    "source": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "7800000"
    },
    "spent": {
      "info": {
        "token": {
          "contract_addr": "terra1cyd63pk2wuvjkqmhlvp9884z4h89rqtn8w8xgz9m28hjd2kzj2cq076xfe"
        }
      },
      "amount": "200000"
    },
    "target": {
      "info": {
        "native_token": {
          "denom": "uluna"
        }
      },
      "amount": "199297"
    },
    "tip": {
      "info": {
        "native_token": {
          "denom": "uluna"
        }
      },
      "amount": "4800000"
    },
    "gas": {
      "info": {
        "native_token": {
          "denom": "uluna"
        }
      },
      "amount": "1000000"
    },
    "last_purchase": 1660596085
  }
}
```
