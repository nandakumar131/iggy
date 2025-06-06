/* Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use crate::BytesSerializable;
use crate::Identifier;
use crate::Sizeable;
use crate::Validatable;
use crate::error::IggyError;
use crate::{Command, DELETE_CONSUMER_OFFSET_CODE};
use crate::{Consumer, ConsumerKind};
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// `DeleteConsumerOffset` command deletes the offset of a consumer from a given partition on the server.
/// It has additional payload:
/// - `consumer` - the consumer that is deleting the offset, either the regular consumer or the consumer group.
/// - `stream_id` - unique stream ID (numeric or name).
/// - `topic_id` - unique topic ID (numeric or name).
/// - `partition_id` - partition ID on which the offset is stored. Has to be specified for the regular consumer. For consumer group it is ignored (use `None`).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DeleteConsumerOffset {
    /// The consumer that is storing the offset, either the regular consumer or the consumer group.
    #[serde(skip)]
    pub consumer: Consumer,
    /// Unique stream ID (numeric or name).
    #[serde(skip)]
    pub stream_id: Identifier,
    /// Unique topic ID (numeric or name).
    #[serde(skip)]
    pub topic_id: Identifier,
    /// Partition ID on which the offset is stored. Has to be specified for the regular consumer. For consumer group it is ignored (use `None`).
    pub partition_id: Option<u32>,
}

impl Default for DeleteConsumerOffset {
    fn default() -> Self {
        DeleteConsumerOffset {
            consumer: Consumer::default(),
            stream_id: Identifier::default(),
            topic_id: Identifier::default(),
            partition_id: Some(1),
        }
    }
}

impl Command for DeleteConsumerOffset {
    fn code(&self) -> u32 {
        DELETE_CONSUMER_OFFSET_CODE
    }
}

impl Validatable<IggyError> for DeleteConsumerOffset {
    fn validate(&self) -> Result<(), IggyError> {
        Ok(())
    }
}

impl BytesSerializable for DeleteConsumerOffset {
    fn to_bytes(&self) -> Bytes {
        let consumer_bytes = self.consumer.to_bytes();
        let stream_id_bytes = self.stream_id.to_bytes();
        let topic_id_bytes = self.topic_id.to_bytes();
        let mut bytes = BytesMut::with_capacity(
            12 + consumer_bytes.len() + stream_id_bytes.len() + topic_id_bytes.len(),
        );
        bytes.put_slice(&consumer_bytes);
        bytes.put_slice(&stream_id_bytes);
        bytes.put_slice(&topic_id_bytes);
        if let Some(partition_id) = self.partition_id {
            bytes.put_u32_le(partition_id);
        } else {
            bytes.put_u32_le(0);
        }
        bytes.freeze()
    }

    fn from_bytes(bytes: Bytes) -> Result<DeleteConsumerOffset, IggyError> {
        if bytes.len() < 15 {
            return Err(IggyError::InvalidCommand);
        }

        let mut position = 0;
        let consumer_kind = ConsumerKind::from_code(bytes[0])?;
        let consumer_id = Identifier::from_bytes(bytes.slice(1..))?;
        position += 1 + consumer_id.get_size_bytes().as_bytes_usize();
        let consumer = Consumer {
            kind: consumer_kind,
            id: consumer_id,
        };
        let stream_id = Identifier::from_bytes(bytes.slice(position..))?;
        position += stream_id.get_size_bytes().as_bytes_usize();
        let topic_id = Identifier::from_bytes(bytes.slice(position..))?;
        position += topic_id.get_size_bytes().as_bytes_usize();
        let partition_id = u32::from_le_bytes(
            bytes[position..position + 4]
                .try_into()
                .map_err(|_| IggyError::InvalidNumberEncoding)?,
        );
        let partition_id = if partition_id == 0 {
            None
        } else {
            Some(partition_id)
        };
        let command = DeleteConsumerOffset {
            consumer,
            stream_id,
            topic_id,
            partition_id,
        };
        Ok(command)
    }
}

impl Display for DeleteConsumerOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.consumer,
            self.stream_id,
            self.topic_id,
            self.partition_id.unwrap_or(0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = DeleteConsumerOffset {
            consumer: Consumer::new(Identifier::numeric(1).unwrap()),
            stream_id: Identifier::numeric(2).unwrap(),
            topic_id: Identifier::numeric(3).unwrap(),
            partition_id: Some(4),
        };

        let bytes = command.to_bytes();
        let mut position = 0;
        let consumer_kind = ConsumerKind::from_code(bytes[0]).unwrap();
        let consumer_id = Identifier::from_bytes(bytes.slice(1..)).unwrap();
        position += 1 + consumer_id.get_size_bytes().as_bytes_usize();
        let consumer = Consumer {
            kind: consumer_kind,
            id: consumer_id,
        };
        let stream_id = Identifier::from_bytes(bytes.slice(position..)).unwrap();
        position += stream_id.get_size_bytes().as_bytes_usize();
        let topic_id = Identifier::from_bytes(bytes.slice(position..)).unwrap();
        position += topic_id.get_size_bytes().as_bytes_usize();
        let partition_id = u32::from_le_bytes(bytes[position..position + 4].try_into().unwrap());

        assert!(!bytes.is_empty());
        assert_eq!(consumer, command.consumer);
        assert_eq!(stream_id, command.stream_id);
        assert_eq!(topic_id, command.topic_id);
        assert_eq!(Some(partition_id), command.partition_id);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let consumer = Consumer::new(Identifier::numeric(1).unwrap());
        let stream_id = Identifier::numeric(2).unwrap();
        let topic_id = Identifier::numeric(3).unwrap();
        let partition_id = 4u32;

        let consumer_bytes = consumer.to_bytes();
        let stream_id_bytes = stream_id.to_bytes();
        let topic_id_bytes = topic_id.to_bytes();
        let mut bytes = BytesMut::with_capacity(
            12 + consumer_bytes.len() + stream_id_bytes.len() + topic_id_bytes.len(),
        );
        bytes.put_slice(&consumer_bytes);
        bytes.put_slice(&stream_id_bytes);
        bytes.put_slice(&topic_id_bytes);
        bytes.put_u32_le(partition_id);

        let command = DeleteConsumerOffset::from_bytes(bytes.freeze());
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.consumer, consumer);
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.topic_id, topic_id);
        assert_eq!(command.partition_id, Some(partition_id));
    }
}
