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

extension IrohaDataModel {
    public indirect enum Value: Swift.Codable {
        
        case u32(UInt32)
        case bool(Bool)
        case string(String)
        case fixed(IrohaDataModelFixed.Fixed)
        case vec([IrohaDataModel.Value])
        case id(IrohaDataModel.IdBox)
        case identifiable(IrohaDataModel.IdentifiableBox)
        case publicKey(IrohaCrypto.PublicKey)
        case parameter(IrohaDataModel.Parameter)
        case signatureCheckCondition(IrohaDataModelAccount.SignatureCheckCondition)
        case transactionValue(IrohaDataModelTransaction.TransactionValue)
        case permissionToken(IrohaDataModelPermissions.PermissionToken)
        
        // MARK: - For Codable purpose
        
        static func discriminant(of case: Self) -> UInt8 {
            switch `case` {
                case .u32:
                    return 20
                case .bool:
                    return 1
                case .string:
                    return 2
                case .fixed:
                    return 3
                case .vec:
                    return 4
                case .id:
                    return 8
                case .identifiable:
                    return 6
                case .publicKey:
                    return 7
                case .parameter:
                    return 8
                case .signatureCheckCondition:
                    return 9
                case .transactionValue:
                    return 10
                case .permissionToken:
                    return 11
            }
        }
        
        // MARK: - Decodable
        
        public init(from decoder: Swift.Decoder) throws {
            var container = try decoder.unkeyedContainer()
            let discriminant = try container.decode(UInt8.self)
            switch discriminant {
            case 0:
                let val0 = try container.decode(UInt32.self)
                self = .u32(val0)
                break
            case 1:
                let val0 = try container.decode(Bool.self)
                self = .bool(val0)
                break
            case 2:
                let val0 = try container.decode(String.self)
                self = .string(val0)
                break
            case 3:
                let val0 = try container.decode(IrohaDataModelFixed.Fixed.self)
                self = .fixed(val0)
                break
            case 4:
                let val0 = try container.decode([IrohaDataModel.Value].self)
                self = .vec(val0)
                break
            case 5:
                let val0 = try container.decode(IrohaDataModel.IdBox.self)
                self = .id(val0)
                break
            case 6:
                let val0 = try container.decode(IrohaDataModel.IdentifiableBox.self)
                self = .identifiable(val0)
                break
            case 7:
                let val0 = try container.decode(IrohaCrypto.PublicKey.self)
                self = .publicKey(val0)
                break
            case 8:
                let val0 = try container.decode(IrohaDataModel.Parameter.self)
                self = .parameter(val0)
                break
            case 9:
                let val0 = try container.decode(IrohaDataModelAccount.SignatureCheckCondition.self)
                self = .signatureCheckCondition(val0)
                break
            case 10:
                let val0 = try container.decode(IrohaDataModelTransaction.TransactionValue.self)
                self = .transactionValue(val0)
                break
            case 11:
                let val0 = try container.decode(IrohaDataModelPermissions.PermissionToken.self)
                self = .permissionToken(val0)
                break
            default:
                throw Swift.DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown discriminant \(discriminant)")
            }
        }
        
        // MARK: - Encodable
        
        public func encode(to encoder: Swift.Encoder) throws {
            var container = encoder.unkeyedContainer()
            try container.encode(Value.discriminant(of: self))
            switch self {
            case let .u32(val0):
                try container.encode(val0)
                break
            case let .bool(val0):
                try container.encode(val0)
                break
            case let .string(val0):
                try container.encode(val0)
                break
            case let .fixed(val0):
                try container.encode(val0)
                break
            case let .vec(val0):
                try container.encode(val0)
                break
            case let .id(val0):
                try container.encode(val0)
                break
            case let .identifiable(val0):
                try container.encode(val0)
                break
            case let .publicKey(val0):
                try container.encode(val0)
                break
            case let .parameter(val0):
                try container.encode(val0)
                break
            case let .signatureCheckCondition(val0):
                try container.encode(val0)
                break
            case let .transactionValue(val0):
                try container.encode(val0)
                break
            case let .permissionToken(val0):
                try container.encode(val0)
                break
            }
        }
    }
}

extension IrohaDataModel.Value: ScaleCodec.Encodable {
    public func encode<E>(in encoder: inout E) throws where E : ScaleCodec.Encoder {
        try encoder.encode(Self.discriminant(of: self))
        switch self {
        case let .u32(val0):
            try encoder.encode(val0)
            break
        case let .id(val0):
            try encoder.encode(val0)
            break
        default:
            // todo: доделать
            break
        }
    }
}