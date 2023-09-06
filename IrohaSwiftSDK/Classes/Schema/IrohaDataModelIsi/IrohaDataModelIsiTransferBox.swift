//
// Copyright 2021 Soramitsu Co., Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

import Foundation
import IrohaSwiftScale
import ScaleCodec

extension IrohaDataModelIsi {
    public struct TransferBox: Swift.Codable {
        
        public var sourceId: IrohaDataModelExpression.EvaluatesTo
        public var object: IrohaDataModelExpression.EvaluatesTo
        public var destinationId: IrohaDataModelExpression.EvaluatesTo
        
        public init(
            sourceId: IrohaDataModelExpression.EvaluatesTo, 
            object: IrohaDataModelExpression.EvaluatesTo, 
            destinationId: IrohaDataModelExpression.EvaluatesTo
        ) {
            self.sourceId = sourceId
            self.object = object
            self.destinationId = destinationId
        }
    }
}

extension IrohaDataModelIsi.TransferBox: ScaleCodec.Encodable {
    public func encode<E>(in encoder: inout E) throws where E : ScaleCodec.Encoder {
        try encoder.encode(sourceId)
        try encoder.encode(object)
        try encoder.encode(destinationId)
    }
}