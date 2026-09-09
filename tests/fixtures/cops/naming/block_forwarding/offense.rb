def foo(&block)
        ^^^^^^ Naming/BlockForwarding: Use anonymous block forwarding.
  bar(&block)
      ^^^^^^ Naming/BlockForwarding: Use anonymous block forwarding.
end

def with_attachable_zip_path(attachable, &block)
                                         ^^^^^^ Naming/BlockForwarding: Use anonymous block forwarding.
  with_io_copied_zip_path(io, &block)
                              ^^^^^^ Naming/BlockForwarding: Use anonymous block forwarding.
end
