require 'colorize'
require 'tmpdir'

def foo
  _x = 1
  _y = 2
  _z = 3
end

class Bar
  _a = 1
  _b = 2
end

module Baz
  CONST = 1
  OTHER = 2
end

def single; end

if cond
  func1
  func2
end

if a1
  b1
elsif a2
  b2
else
  c
end

unless cond
  func1
  func2
end

case a
when b
  c
  c
when d
else
  f
end

while cond
  func1
  func2
end

until cond
  func1
  func2
end

for _var in 1..10
  func1
  func2
end

begin
  func1
  func2
end

module VkontakteApi
  class Method
    def call(args = {}, &block)
      response = API.call(full_name, args, token)
      Result.process(response, type, block)
    end

  private
    def full_name
      parts = [@previous_resolver.name, @name].compact.map { |part| camelize(part) }
      parts.join(".").gsub(/[^A-Za-z.]/, "")
    end
  end
end

class A
  def _to_s(key)
    foo
  end; protected :_to_s

  def to_plain_s; _to_s(:a); end
end

def foo
  pnode =
    @node; loop do
      pnode = parent_node(pnode)
      break
    end
end

while a
end

for _var in 1..10
end

if a
else
end

require 'ostruct'

module ClinicFinder
  module Modules
    module GestationHelper; end
  end
end

if RUBY_VERSION < '1.9'
    def initialize
    end

    def inspect
    end

  private

    def sync_keys!
    end

  end

def erb(title:)
  _out = +''
  _out << title
  _out
end

# Class reopen with leading comments before the first method (peatio FP)
class ActiveRecord::Relation
  # Allow passing index hints to MySQL.
  #
  # Example:
  #   Message.first.events.use_index(:idx)
  #
  def use_index(index_name)
    from("#{quoted_table_name} USE INDEX (#{index_name})")
  end
end

class Grape::Entity::Exposure::Base
  def documentation
    @documentation.respond_to?(:call) ? @documentation.call : @documentation
  end
end

class AllMailInterceptor
  def self.delivering_email(message)
    message.perform_deliveries = true
  end
end

class Rack::Session::Redis
  def set_session(env, session_id, new_session, options)
    with_lock(env, false) do
      with do |c|
        c.set(session_id, new_session, options)
      end
      session_id
    end
  end
end

module I18n
  module Backend
    class Simple
      def reload!
        super
        return if ENV['RAILS_ENV'] == 'test'
        result = call
        if result
          log('ok')
        else
          log('fail')
        end
      end
    end
  end

  def self.use_translation_with_tooltip(toggle_flag)
    if toggle_flag
      def self.t(key = nil)
        translate(key)
      end
    end
    yield
  ensure
    if toggle_flag
      def self.t(key = nil)
        translate(key)
      end
    end
  end
end
