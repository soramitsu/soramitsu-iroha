/**
 * Copyright Soramitsu Co., Ltd. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#ifndef FAKE_PEER_ODOS_NETWORK_NOTIFIER_HPP_
#define FAKE_PEER_ODOS_NETWORK_NOTIFIER_HPP_

#include <rxcpp/rx-lite.hpp>

#include <mutex>

#include "consensus/round.hpp"
#include "framework/integration_framework/fake_peer/types.hpp"
#include "ordering/on_demand_ordering_service.hpp"

namespace integration_framework {
  namespace fake_peer {

    class OnDemandOsNetworkNotifier final
        : public iroha::ordering::OnDemandOrderingService {
     public:
      OnDemandOsNetworkNotifier(const std::shared_ptr<FakePeer> &fake_peer);

      void onBatches(CollectionType batches) override;

      boost::optional<std::shared_ptr<const ProposalType>> onRequestProposal(
          iroha::consensus::Round round) override;

      void onCollaborationOutcome(iroha::consensus::Round round) override;

      void onTxsCommitted(const HashesSetType &hashes) override;

      void forCachedBatches(
          std::function<void(
              const iroha::ordering::transport::OdOsNotification::BatchesSetType
                  &)> const &f) override;

      bool isEmptyBatchesCache() const override;

      rxcpp::observable<iroha::consensus::Round>
      getProposalRequestsObservable();

      rxcpp::observable<std::shared_ptr<BatchesCollection>>
      getBatchesObservable();

     private:
      std::weak_ptr<FakePeer> fake_peer_wptr_;
      rxcpp::subjects::subject<iroha::consensus::Round> rounds_subject_;
      std::mutex rounds_subject_mutex_;
      rxcpp::subjects::subject<std::shared_ptr<BatchesCollection>>
          batches_subject_;
      std::mutex batches_subject_mutex_;
    };

  }  // namespace fake_peer
}  // namespace integration_framework

#endif /* FAKE_PEER_ODOS_NETWORK_NOTIFIER_HPP_ */
