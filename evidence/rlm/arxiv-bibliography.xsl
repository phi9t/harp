<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="text" encoding="UTF-8"/>
  <xsl:param name="source_id"/>

  <xsl:template match="/">
    <xsl:for-each select="//*[local-name()='li' and contains(concat(' ', normalize-space(@class), ' '), ' ltx_bibitem ')]">
      <xsl:value-of select="$source_id"/>
      <xsl:text>&#9;</xsl:text>
      <xsl:value-of select="@id"/>
      <xsl:text>&#9;</xsl:text>
      <xsl:value-of select="normalize-space(string(.))"/>
      <xsl:text>&#9;</xsl:text>
      <xsl:value-of select="(.//*[local-name()='a']/@href)[1]"/>
      <xsl:text>&#9;arxiv-html:ltx_bibitem&#10;</xsl:text>
    </xsl:for-each>
  </xsl:template>
</xsl:stylesheet>
