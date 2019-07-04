// Smart contract de teste baseado no exemplo ERC20 do ink!
// O contrato segue o padrão ERC20 do Ethereum.
// O saldo dos usuários são armazenados num HashMap.
// Funções foram criadas para transferir fundos e permitir que uma terceira pessoa transfira tokens em seu nome (allowance). 

#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::{
    memory::format,
    storage,
    env::DefaultSrmlTypes,
};
use ink_lang::contract;
use ink_model::EnvHandler;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    // Evento para quando uma transferência de tokens ocorre
    event Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance,
    }

    // Evento para quando um uso por terceiros ocorre
    event Approval {
        owner: AccountId,
        spender: AccountId,
        value: Balance,
    }

    // Um storage típico do ERC20
    struct Teste {
        // Quantidade total de tokens
        total_supply: storage::Value<Balance>,
        // Saldo de cada usuário.
        balances: storage::HashMap<AccountId, Balance>,
        // Saldo que pode ser gasto por terceiros: (owner, spender) -> allowed
        allowances: storage::HashMap<(AccountId, AccountId), Balance>,
    }

    // Será executado no deploy do contrato (apenas uma vez)
    impl Deploy for Teste {
        fn deploy(&mut self, init_value: Balance) {
            self.total_supply.set(init_value);
            self.balances.insert(env.caller(), init_value);
            env.emit(Transfer {
                from: None,
                to: Some(env.caller()),
                value: init_value
            });
        }
    }

    // Funções públicas
    impl Teste {
        // Retorna o número total de tokens existentes e imprime no terminal para efeito de debug
        pub(external) fn total_supply(&self) -> Balance {
            let total_supply = *self.total_supply;
            env.println(&format!("Teste::total_supply = {:?}", total_supply));
            total_supply
        }

        // Retorna o saldo de uma certa AccountId chamada owner e imprime no terminal para efeito de debug
        pub(external) fn balance_of(&self, owner: AccountId) -> Balance {
            let balance = self.balance_of_or_zero(&owner);
            env.println(&format!("Teste::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        // Retorna a quantidade de tokens que um owner emprestou para um spender e imprime no terminal para efeito de debug
        pub(external) fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            let allowance = self.allowance_or_zero(&owner, &spender);
            env.println(&format!(
                "Teste::allowance(owner = {:?}, spender = {:?}) = {:?}",
                owner, spender, allowance
            ));
            allowance
        }

        // Transfere (value) tokens do "sender" (env.caller()) para o "to" (AccountId)
        // Devolve booleano de acordo com o sucesso da transação
        pub(external) fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            self.transfer_impl(env, env.caller(), to, value)
        }

        // Aprova o "spender" (AccountId) a gastar (value) tokens em nome de quem manda a mensagem (owner)
        // Devolve booleano de acordo com o sucesso da transação
        pub(external) fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            let owner = env.caller();
            self.allowances.insert((owner, spender), value);
            env.emit(Approval {
                owner: owner,
                spender: spender,
                value: value
            });
            true
        }

        // Transfere (value) tokens de "from" (AccountId) para "to" (AccountId)
        // Devolve booleano de acordo com o sucesso da transação
        pub(external) fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            let allowance = self.allowance_or_zero(&from, &env.caller());
            if allowance < value {
                return false
            }
            self.allowances.insert((from, env.caller()), allowance - value);
            self.transfer_impl(env, from, to, value)
        }
    }

    // Funções privadas
    impl Teste {
        // Retorna o saldo de "of" (AccountId) ou 0 se não houver saldo
        fn balance_of_or_zero(&self, of: &AccountId) -> Balance {
            *self.balances.get(of).unwrap_or(&0)
        }

        // Retorna o allowance ou 0 se não existir
        fn allowance_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }

        // Transfere tokens de "from" (AccountId) para "to" (AccountId)
        // Devolve booleano de acordo com o sucesso da transação
        fn transfer_impl(&mut self, env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, from: AccountId, to: AccountId, value: Balance) -> bool {
            let balance_from = self.balance_of_or_zero(&from);
            let balance_to = self.balance_of_or_zero(&to);
            if balance_from < value {
                return false
            }
            self.balances.insert(from, balance_from - value);
            self.balances.insert(to, balance_to + value);
            env.emit(Transfer {
                from: Some(from),
                to: Some(to),
                value: value
            });
            true
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn deployment_works() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);

        // Deploy do contrato com valor inicial (init_value)
        let teste = Teste::deploy_mock(1234);
        // Checa se total_supply é igual a init_value
        assert_eq!(teste.total_supply(), 1234);
        // Checa se o balance_of da Alice é igual a init_value
        assert_eq!(teste.balance_of(alice), 1234);
    }

    #[test]
    fn transfer_works() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);

        env::test::set_caller::<Types>(alice);
        // Deploy do contrato com valor inicial (init_value)
        let mut teste = Teste::deploy_mock(1234);
        // Alice não tem tokens o suficiente:
        assert_eq!(teste.transfer(bob, 4321), false);
        // Mas Alice pode fazer isso:
        assert_eq!(teste.transfer(bob, 234), true);
        // Checa se Alice e Bob tem os saldos corretos
        assert_eq!(teste.balance_of(alice), 1000);
        assert_eq!(teste.balance_of(bob), 234);
    }

    #[test]
    fn allowance_works() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);
        let charlie = AccountId::from([0x2; 32]);

        env::test::set_caller::<Types>(alice);
        // Deploy do contrato com valor inicial (init_value)
        let mut teste = Teste::deploy_mock(1234);
        // Bob não tem allowance do saldo de Alice
        assert_eq!(teste.allowance(alice, bob), 0);
        // Então, Bob não pode transferir tokens de dentro da conta da Alice
        env::test::set_caller::<Types>(bob);
        assert_eq!(teste.transfer_from(alice, bob, 1), false);
        // Alice pode aprovar o uso de uma porção do seu saldo para Bob
        env::test::set_caller::<Types>(alice);
        assert_eq!(teste.approve(bob, 20), true);
        // E então a allowance será permitida
        assert_eq!(teste.allowance(alice, bob), 20);
        // Charlie não pode enviar em nome de Bob
        env::test::set_caller::<Types>(charlie);
        assert_eq!(teste.transfer_from(alice, bob, 10), false);
        // Bob não pode transferir mais do que lhe é permitido
        env::test::set_caller::<Types>(bob);
        assert_eq!(teste.transfer_from(alice, charlie, 25), false);
        // Mas uma pequena quantia funciona
        assert_eq!(teste.transfer_from(alice, charlie, 10), true);
        // Checa se a allowance está atualizada
        assert_eq!(teste.allowance(alice, bob), 10);
        // E que o saldo foi transferido para a pessoa correta
        assert_eq!(teste.balance_of(charlie), 10);
    }

    #[test]
    fn events_work() {
        let alice = AccountId::from([0x0; 32]);
        let bob = AccountId::from([0x1; 32]);

        // Nenhum evento no começo
        env::test::set_caller::<Types>(alice);
        assert_eq!(env::test::emitted_events::<Types>().count(), 0);
        // Um evento foi emitido inicialmente
        let mut teste = Teste::deploy_mock(1234);
        assert_eq!(env::test::emitted_events::<Types>().count(), 1);
        // Eventos são emitidos no caso de aprovações
        assert_eq!(teste.approve(bob, 20), true);
        assert_eq!(env::test::emitted_events::<Types>().count(), 2);
        // Eventos são emitidos no caso de transferências
        assert_eq!(teste.transfer(bob, 10), true);
        assert_eq!(env::test::emitted_events::<Types>().count(), 3);
    }
}
