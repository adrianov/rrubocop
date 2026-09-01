task :foo do
^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  puts "hello"
end

task :bar do
^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  User.all.each { |u| puts u.name }
end

task :cleanup do
^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  OldRecord.delete_all
end

task 'generate_report' do
^^^^^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  Report.generate
end

task('update_cache') { Cache.refresh }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.

task migrate: [] do
^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  ActiveRecord::Base.connection.migrate
end

task refresh: [] do
^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  Cache.clear
end

task name do
^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  puts "local variable task name"
end

task(a.to_sym) { puts "method call task name" }
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.

task short_name do
^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  run_command
end

task :release, :rel, :reuse, :reltest, :needs => [:prerelease] do |t, args|
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  puts "release"
end

task :update_version, :rel, :reuse, :needs => [:prerelease] do |t, args|
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  puts "update"
end

task :tag, :rel, :needs => [:prerelease] do |t, args|
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/RakeEnvironment: Include `:environment` task as a dependency for all Rake tasks.
  puts "tag"
end
