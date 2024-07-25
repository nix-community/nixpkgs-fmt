{ lib, hello }:
{
  hello-insecure = hello.overrideAttrs (oldAttrs: {
    meta = oldAttrs.meta // {
      /* Mark as insecure ⁧⁦/*⁩ ⁦insecure = true;⁩ /*⁩ 2000-00-00 */
    };
  });
}
