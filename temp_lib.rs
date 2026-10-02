                        layout.parts.len(),
                        array_len
                    ));
                }
                let stride = match array_len {
                    0 => 0,
                    1 => layout.size,
                    _ => layout.parts[1].offset.checked_sub(layout.parts[0].offset).ok_or_else(
                        || {
                            format!(
                                "Codegen error: {} array element offsets are not monotonic",
                                context
                            )
                        },
                    )?,
                };

                let index_value = match emitter.storage.get(index_local) {
                    Some(NativeStorage::Scalar(variable)) => builder.use_var(*variable),
                    _ => {
                        return Err(format!(
                            "Codegen error: {} dynamic index local {:?} has no scalar storage",