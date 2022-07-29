//! `World`-related ISI implementations.

use iroha_telemetry::metrics;

use super::prelude::*;
use crate::prelude::*;

/// Iroha Special Instructions that have `World` as their target.
pub mod isi {
    use eyre::Result;
    use iroha_data_model::prelude::*;

    use super::*;

    impl Execute for Register<Peer> {
        type Error = Error;

        #[metrics(+"register_peer")]
        fn execute(
            self,
            _authority: <Account as Identifiable>::Id,
            wsv: &WorldStateView,
        ) -> Result<(), Self::Error> {
            let peer_id = self.object.id;

            wsv.modify_world(|world| {
                if !world.trusted_peers_ids.insert(peer_id.clone()) {
                    return Err(Error::Repetition(
                        InstructionType::Register,
                        IdBox::PeerId(peer_id),
                    ));
                }

                Ok(PeerEvent::Added(peer_id).into())
            })
        }
    }

    impl Execute for Unregister<Peer> {
        type Error = Error;

        #[metrics(+"unregister_peer")]
        fn execute(
            self,
            _authority: <Account as Identifiable>::Id,
            wsv: &WorldStateView,
        ) -> Result<(), Self::Error> {
            let peer_id = self.object_id;
            wsv.modify_world(|world| {
                if world.trusted_peers_ids.remove(&peer_id).is_none() {
                    return Err(FindError::Peer(peer_id).into());
                }

                Ok(PeerEvent::Removed(peer_id).into())
            })
        }
    }

    impl Execute for Register<Domain> {
        type Error = Error;

        #[metrics("register_domain")]
        fn execute(
            self,
            _authority: <Account as Identifiable>::Id,
            wsv: &WorldStateView,
        ) -> Result<(), Self::Error> {
            let domain: Domain = self.object.build();
            let domain_id = domain.id().clone();

            domain_id
                .name
                .validate_len(wsv.config.ident_length_limits)
                .map_err(Error::Validate)?;

            wsv.modify_world(|world| {
                if world.domains.contains_key(&domain_id) {
                    return Err(Error::Repetition(
                        InstructionType::Register,
                        IdBox::DomainId(domain_id),
                    ));
                }

                world.domains.insert(domain_id.clone(), domain);
                Ok(DomainEvent::Created(domain_id).into())
            })?;

            wsv.metrics.domains.inc();
            Ok(())
        }
    }

    impl Execute for Unregister<Domain> {
        type Error = Error;

        #[metrics("unregister_domain")]
        fn execute(
            self,
            _authority: <Account as Identifiable>::Id,
            wsv: &WorldStateView,
        ) -> Result<(), Self::Error> {
            let domain_id = self.object_id;

            wsv.modify_world(|world| {
                if world.domains.remove(&domain_id).is_none() {
                    return Err(FindError::Domain(domain_id).into());
                }

                Ok(DomainEvent::Deleted(domain_id).into())
            })?;

            wsv.metrics.domains.dec();
            Ok(())
        }
    }

    impl Execute for Register<Role> {
        type Error = Error;

        #[metrics(+"register_role")]
        fn execute(
            self,
            _authority: <Account as Identifiable>::Id,
            wsv: &WorldStateView,
        ) -> Result<(), Self::Error> {
            let role = self.object.build();

            wsv.modify_world(|world| {
                let role_id = role.id().clone();

                if world.roles.contains_key(&role_id) {
                    return Err(Error::Repetition(
                        InstructionType::Register,
                        IdBox::RoleId(role_id),
                    ));
                }

                world.roles.insert(role_id.clone(), role);
                Ok(RoleEvent::Created(role_id).into())
            })
        }
    }

    impl Execute for Unregister<Role> {
        type Error = Error;

        #[metrics("unregister_role")]
        fn execute(
            self,
            _authority: <Account as Identifiable>::Id,
            wsv: &WorldStateView,
        ) -> Result<(), Self::Error> {
            let role_id = self.object_id;

            let mut accounts_with_role = vec![];
            for domain in wsv.domains().iter() {
                let account_ids = domain.accounts().filter_map(|account| {
                    if account.contains_role(&role_id) {
                        return Some(account.id().clone());
                    }

                    None
                });

                accounts_with_role.extend(account_ids);
            }

            for account_id in accounts_with_role {
                wsv.modify_account(&account_id.clone(), |account| {
                    if !account.remove_role(&role_id) {
                        error!(%role_id, "role not found - this is a bug");
                    }

                    Ok(AccountEvent::RoleRevoked(account_id))
                })?;
            }

            wsv.modify_world(|world| {
                if world.roles.remove(&role_id).is_none() {
                    return Err(FindError::Role(role_id).into());
                }

                Ok(RoleEvent::Deleted(role_id).into())
            })
        }
    }
}

/// Query module provides `IrohaQuery` Peer related implementations.
pub mod query {
    use eyre::Result;
    use iroha_data_model::prelude::*;

    use super::*;
    use crate::smartcontracts::query::Error;

    impl ValidQuery for FindAllRoles {
        #[metrics(+"find_all_roles")]
        fn execute(&self, wsv: &WorldStateView) -> Result<Self::Output, Error> {
            Ok(wsv
                .world
                .roles
                .iter()
                .map(|role| role.value().clone())
                .collect())
        }
    }

    impl ValidQuery for FindAllRoleIds {
        #[metrics(+"find_all_role_ids")]
        fn execute(&self, wsv: &WorldStateView) -> Result<Self::Output, Error> {
            Ok(wsv
               .world
               .roles
               .iter()
               // To me, this should probably be a method, not a field.
               .map(|role| role.id().clone())
               .collect())
        }
    }

    impl ValidQuery for FindRoleByRoleId {
        #[metrics(+"find_role_by_role_id")]
        fn execute(&self, wsv: &WorldStateView) -> Result<Self::Output, Error> {
            let role_id = self
                .id
                .evaluate(wsv, &Context::new())
                .map_err(|e| Error::Evaluate(e.to_string()))?;
            iroha_logger::trace!(%role_id);

            wsv.world.roles.get(&role_id).map_or_else(
                || Err(Error::Find(Box::new(FindError::Role(role_id)))),
                |role_ref| Ok(role_ref.clone()),
            )
        }
    }

    impl ValidQuery for FindAllPeers {
        #[metrics("find_all_peers")]
        fn execute(&self, wsv: &WorldStateView) -> Result<Self::Output, Error> {
            Ok(wsv.peers())
        }
    }
}
