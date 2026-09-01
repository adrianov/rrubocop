class Base
  def shops_stocks_template = raise NotImplementedError
  def start_log = logger.info 'sync'
end
