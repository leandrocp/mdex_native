defmodule MDExNative.Sanitize do
  @moduledoc false

  def normalize(nil), do: nil
  def normalize(:default), do: :clean
  def normalize(:clean), do: :clean
  def normalize({:custom, _custom} = sanitize), do: sanitize
  def normalize(options) when is_list(options), do: adapt_options(options)

  defp adapt_options(options) do
    {:custom,
     %{
       link_rel: options[:link_rel],
       tags: %{
         set: options[:tags],
         add: options[:add_tags],
         rm: options[:rm_tags]
       },
       clean_content_tags: %{
         set: options[:clean_content_tags],
         add: options[:add_clean_content_tags],
         rm: options[:rm_clean_content_tags]
       },
       tag_attributes: %{
         set: options[:tag_attributes],
         add: options[:add_tag_attributes],
         rm: options[:rm_tag_attributes]
       },
       tag_attribute_values: %{
         set: options[:tag_attribute_values],
         add: options[:add_tag_attribute_values],
         rm: options[:rm_tag_attribute_values]
       },
       set_tag_attribute_values: %{
         set: options[:set_tag_attribute_values],
         add: options[:set_tag_attribute_value],
         rm: options[:rm_set_tag_attribute_value]
       },
       generic_attribute_prefixes: %{
         set: options[:generic_attribute_prefixes],
         add: options[:add_generic_attribute_prefixes],
         rm: options[:rm_generic_attribute_prefixes]
       },
       generic_attributes: %{
         set: options[:generic_attributes],
         add: options[:add_generic_attributes],
         rm: options[:rm_generic_attributes]
       },
       url_schemes: %{
         set: options[:url_schemes],
         add: options[:add_url_schemes],
         rm: options[:rm_url_schemes]
       },
       url_relative: options[:url_relative],
       allowed_classes: %{
         set: options[:allowed_classes],
         add: options[:add_allowed_classes],
         rm: options[:rm_allowed_classes]
       },
       strip_comments: options[:strip_comments],
       id_prefix: options[:id_prefix],
       filter_style_properties: options[:filter_style_properties]
     }}
  end
end
